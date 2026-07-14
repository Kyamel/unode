use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use thiserror::Error;

pub type MessageValue = JsonValue;
pub type MessageValues = BTreeMap<String, MessageValue>;
pub type MessageCatalog = BTreeMap<String, MessageEntry>;
pub type MessageCatalogs = BTreeMap<String, MessageCatalog>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MessageEntry {
    Text(String),
    Group(MessageCatalog),
}

#[derive(Debug, Clone, PartialEq)]
pub enum I18nText {
    Literal(String),
    Key {
        key: String,
        values: MessageValues,
        fallback: Option<String>,
    },
}

impl From<String> for I18nText {
    fn from(value: String) -> Self {
        Self::Literal(value)
    }
}

impl From<&str> for I18nText {
    fn from(value: &str) -> Self {
        Self::Literal(value.to_string())
    }
}

pub fn msg(key: impl Into<String>) -> I18nText {
    I18nText::Key {
        key: key.into(),
        values: BTreeMap::new(),
        fallback: None,
    }
}

pub fn msg_with(
    key: impl Into<String>,
    values: MessageValues,
    fallback: impl Into<Option<String>>,
) -> I18nText {
    I18nText::Key {
        key: key.into(),
        values,
        fallback: fallback.into(),
    }
}

pub trait LocaleSource: Send + Sync {
    fn current_locale(&self) -> String;
}

impl<F> LocaleSource for F
where
    F: Fn() -> String + Send + Sync,
{
    fn current_locale(&self) -> String {
        self()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct I18nCatalogRegistrationEvent {
    pub plugin_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locales: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct I18nLookupEvent {
    pub plugin_id: String,
    pub requested_locale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_locale: Option<String>,
    pub key: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub values: MessageValues,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
    pub output: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub missing: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub used_fallback: bool,
}

pub trait I18nInspector: Send + Sync {
    fn on_register(&self, _event: &I18nCatalogRegistrationEvent) {}
    fn on_lookup(&self, event: &I18nLookupEvent);
}

impl<F> I18nInspector for F
where
    F: Fn(&I18nLookupEvent) + Send + Sync,
{
    fn on_lookup(&self, event: &I18nLookupEvent) {
        self(event);
    }
}

#[derive(Debug, Error)]
pub enum I18nError {
    #[error("invalid message catalogs json")]
    InvalidCatalogsJson(#[from] serde_json::Error),
}

#[derive(Clone)]
pub struct PluginI18n {
    plugin_id: Arc<str>,
    locale_source: Arc<dyn LocaleSource>,
    catalogs: Arc<RwLock<MessageCatalogs>>,
    inspectors: Arc<RwLock<Vec<Arc<dyn I18nInspector>>>>,
}

impl std::fmt::Debug for PluginI18n {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginI18n")
            .field("plugin_id", &self.plugin_id)
            .field("supported_locales", &self.supported_locales())
            .finish()
    }
}

impl PluginI18n {
    pub fn new(plugin_id: impl Into<String>, locale_source: Arc<dyn LocaleSource>) -> Self {
        Self {
            plugin_id: Arc::from(plugin_id.into()),
            locale_source,
            catalogs: Arc::new(RwLock::new(BTreeMap::new())),
            inspectors: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_catalogs(
        plugin_id: impl Into<String>,
        locale_source: Arc<dyn LocaleSource>,
        catalogs: MessageCatalogs,
    ) -> Self {
        let i18n = Self::new(plugin_id, locale_source);
        i18n.register(catalogs);
        i18n
    }

    pub fn register(&self, catalogs: MessageCatalogs) {
        let locales = catalogs.keys().cloned().collect::<Vec<_>>();
        *self.catalogs.write().expect("i18n catalogs poisoned") = catalogs;
        self.emit_register(locales);
    }

    pub fn register_locale(&self, locale: impl Into<String>, catalog: MessageCatalog) {
        let locale = locale.into();
        self.catalogs
            .write()
            .expect("i18n catalogs poisoned")
            .insert(locale.clone(), catalog);
        self.emit_register(self.supported_locales());
    }

    pub fn register_json_str(&self, catalogs_json: &str) -> Result<(), I18nError> {
        let catalogs = serde_json::from_str::<MessageCatalogs>(catalogs_json)?;
        self.register(catalogs);
        Ok(())
    }

    pub fn add_inspector(&self, inspector: Arc<dyn I18nInspector>) {
        self.inspectors
            .write()
            .expect("i18n inspectors poisoned")
            .push(inspector);
    }

    pub fn locale(&self) -> String {
        self.locale_source.current_locale()
    }

    pub fn translator(&self) -> PluginTranslator {
        PluginTranslator { i18n: self.clone() }
    }

    pub fn supported_locales(&self) -> Vec<String> {
        self.catalogs
            .read()
            .expect("i18n catalogs poisoned")
            .keys()
            .cloned()
            .collect()
    }

    pub fn catalogs_snapshot(&self) -> MessageCatalogs {
        self.catalogs
            .read()
            .expect("i18n catalogs poisoned")
            .clone()
    }

    pub fn t(&self, key: &str, values: Option<&MessageValues>, fallback: Option<&str>) -> String {
        let requested_locale = self.locale();
        let catalogs = self.catalogs.read().expect("i18n catalogs poisoned");
        let resolved = resolve_template(&catalogs, &requested_locale, key);

        let (resolved_locale, template) = match resolved {
            Some(found) => (Some(found.locale), found.template),
            None => {
                let output = fallback.unwrap_or(key).to_string();
                self.emit_lookup(I18nLookupEvent {
                    plugin_id: self.plugin_id.to_string(),
                    requested_locale,
                    resolved_locale: None,
                    key: key.to_string(),
                    values: values.cloned().unwrap_or_default(),
                    fallback: fallback.map(ToString::to_string),
                    output: output.clone(),
                    missing: true,
                    used_fallback: fallback.is_some(),
                });
                return output;
            }
        };

        let output = interpolate(template, values);
        self.emit_lookup(I18nLookupEvent {
            plugin_id: self.plugin_id.to_string(),
            requested_locale,
            resolved_locale,
            key: key.to_string(),
            values: values.cloned().unwrap_or_default(),
            fallback: fallback.map(ToString::to_string),
            output: output.clone(),
            missing: false,
            used_fallback: false,
        });
        output
    }

    pub fn resolve_text(&self, text: &I18nText) -> String {
        match text {
            I18nText::Literal(value) => value.clone(),
            I18nText::Key { key, values, fallback } => self.t(key, Some(values), fallback.as_deref()),
        }
    }

    fn emit_register(&self, locales: Vec<String>) {
        let event = I18nCatalogRegistrationEvent {
            plugin_id: self.plugin_id.to_string(),
            locales,
        };

        for inspector in self
            .inspectors
            .read()
            .expect("i18n inspectors poisoned")
            .iter()
        {
            inspector.on_register(&event);
        }
    }

    fn emit_lookup(&self, event: I18nLookupEvent) {
        for inspector in self
            .inspectors
            .read()
            .expect("i18n inspectors poisoned")
            .iter()
        {
            inspector.on_lookup(&event);
        }
    }
}

#[derive(Clone, Debug)]
pub struct PluginTranslator {
    i18n: PluginI18n,
}

impl PluginTranslator {
    pub fn locale(&self) -> String {
        self.i18n.locale()
    }

    pub fn t(&self, key: &str, values: Option<&MessageValues>, fallback: Option<&str>) -> String {
        self.i18n.t(key, values, fallback)
    }
}

struct ResolvedTemplate<'a> {
    locale: String,
    template: &'a str,
}

fn resolve_template<'a>(
    catalogs: &'a MessageCatalogs,
    locale: &str,
    key: &str,
) -> Option<ResolvedTemplate<'a>> {
    if let Some((resolved_locale, catalog)) = find_catalog(catalogs, locale) {
        if let Some(template) = lookup_message(catalog, key) {
            return Some(ResolvedTemplate {
                locale: resolved_locale,
                template,
            });
        }
    }

    if let Some((resolved_locale, fallback_catalog)) = find_catalog(catalogs, "en") {
        lookup_message(fallback_catalog, key).map(|template| ResolvedTemplate {
            locale: resolved_locale,
            template,
        })
    } else {
        None
    }
}

fn find_catalog<'a>(catalogs: &'a MessageCatalogs, locale: &str) -> Option<(String, &'a MessageCatalog)> {
    let normalized = normalize_locale(locale);

    if let Some((key, catalog)) = catalogs
        .iter()
        .find(|(key, _)| normalize_locale(key) == normalized)
    {
        return Some((key.clone(), catalog));
    }

    let base = normalized.split('-').next().unwrap_or_default();
    if let Some((key, catalog)) = catalogs.iter().find(|(key, _)| {
        let candidate = normalize_locale(key);
        candidate == base || candidate.starts_with(&format!("{base}-"))
    }) {
        return Some((key.clone(), catalog));
    }

    catalogs
        .iter()
        .find(|(key, _)| normalize_locale(key) == "en")
        .map(|(key, catalog)| (key.clone(), catalog))
        .or_else(|| catalogs.iter().next().map(|(key, catalog)| (key.clone(), catalog)))
}

fn normalize_locale(locale: &str) -> String {
    locale.trim().to_ascii_lowercase()
}

fn lookup_message<'a>(catalog: &'a MessageCatalog, key: &str) -> Option<&'a str> {
    if let Some(MessageEntry::Text(value)) = catalog.get(key) {
        return Some(value.as_str());
    }

    let mut current = catalog;
    let mut segments = key.split('.').peekable();

    while let Some(segment) = segments.next() {
        let entry = current.get(segment)?;
        match entry {
            MessageEntry::Text(value) if segments.peek().is_none() => return Some(value.as_str()),
            MessageEntry::Text(_) => return None,
            MessageEntry::Group(group) => current = group,
        }
    }

    None
}

fn interpolate(template: &str, values: Option<&MessageValues>) -> String {
    let Some(values) = values else {
        return template.to_string();
    };

    let mut out = template.to_string();
    for (key, value) in values {
        let needle = format!("{{{key}}}");
        out = out.replace(&needle, &format_message_value(value));
    }
    out
}

fn format_message_value(value: &MessageValue) -> String {
    match value {
        JsonValue::Null => String::new(),
        JsonValue::Bool(value) => value.to_string(),
        JsonValue::Number(value) => value.to_string(),
        JsonValue::String(value) => value.clone(),
        JsonValue::Array(_) | JsonValue::Object(_) => value.to_string(),
    }
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use serde_json::json;

    use super::{
        msg, msg_with, I18nCatalogRegistrationEvent, I18nInspector, I18nLookupEvent, MessageCatalogs,
        MessageValues, PluginI18n,
    };

    fn sample_catalogs() -> MessageCatalogs {
        serde_json::from_value(json!({
            "en": {
                "nav": {
                    "browse": "Browse {kind}"
                },
                "title": "Hot"
            },
            "pt-BR": {
                "nav": {
                    "browse": "Explorar {kind}"
                }
            }
        }))
        .expect("sample catalogs")
    }

    #[test]
    fn resolves_exact_locale_and_interpolates() {
        let i18n = PluginI18n::with_catalogs("demo.plugin", Arc::new(|| "pt-BR".to_string()), sample_catalogs());
        let mut values = MessageValues::new();
        values.insert("kind".into(), json!("mangas"));

        assert_eq!(i18n.t("nav.browse", Some(&values), None), "Explorar mangas");
    }

    #[test]
    fn falls_back_to_english_when_locale_is_missing() {
        let i18n = PluginI18n::with_catalogs("demo.plugin", Arc::new(|| "es-ES".to_string()), sample_catalogs());
        let mut values = MessageValues::new();
        values.insert("kind".into(), json!("mangas"));

        assert_eq!(i18n.t("nav.browse", Some(&values), None), "Browse mangas");
    }

    #[test]
    fn msg_helper_resolves_lazy_text() {
        let i18n = PluginI18n::with_catalogs("demo.plugin", Arc::new(|| "en".to_string()), sample_catalogs());
        let mut values = MessageValues::new();
        values.insert("kind".into(), json!("works"));

        assert_eq!(i18n.resolve_text(&msg("title")), "Hot");
        assert_eq!(
            i18n.resolve_text(&msg_with("nav.browse", values, Some("fallback".to_string()))),
            "Browse works"
        );
    }

    #[test]
    fn inspectors_receive_registration_and_lookup_events() {
        #[derive(Default)]
        struct Recorder {
            registrations: Mutex<Vec<I18nCatalogRegistrationEvent>>,
            lookups: Mutex<Vec<I18nLookupEvent>>,
        }

        impl I18nInspector for Recorder {
            fn on_register(&self, event: &I18nCatalogRegistrationEvent) {
                self.registrations.lock().expect("registrations").push(event.clone());
            }

            fn on_lookup(&self, event: &I18nLookupEvent) {
                self.lookups.lock().expect("lookups").push(event.clone());
            }
        }

        let recorder = Arc::new(Recorder::default());
        let i18n = PluginI18n::new("demo.plugin", Arc::new(|| "en".to_string()));
        i18n.add_inspector(recorder.clone());
        i18n.register(sample_catalogs());

        assert_eq!(i18n.resolve_text(&msg("title")), "Hot");

        let registrations = recorder.registrations.lock().expect("registrations");
        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].locales, vec!["en".to_string(), "pt-BR".to_string()]);
        drop(registrations);

        let lookups = recorder.lookups.lock().expect("lookups");
        assert_eq!(lookups.len(), 1);
        assert_eq!(lookups[0].resolved_locale.as_deref(), Some("en"));
        assert_eq!(lookups[0].key, "title");
        assert!(!lookups[0].missing);
    }
}
