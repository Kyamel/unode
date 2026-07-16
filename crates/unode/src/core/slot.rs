use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use thiserror::Error;

use crate::core::ast::*;
use crate::core::canonical::*;
use crate::core::normalize::normalize_screen;
use crate::core::resolver::{DefaultExprResolver, ResolverContext};
use crate::core::runtime::{PluginId, PluginManifest, ResolvedRoute, SlotContributionDecl};
use crate::core::state::MemoryStateStore;

pub const SLOT_ORIGIN_PLUGIN_ID: &str = "_originPluginId";
pub const SLOT_ORIGIN_CONTRIBUTION_ID: &str = "_originContributionId";

pub const DEFAULT_MAX_SLOT_DEPTH: usize = 16;
pub const DEFAULT_MAX_SLOT_CONTRIBUTIONS_PER_SLOT: usize = 128;
pub const DEFAULT_MAX_SLOT_NODES_PER_CONTRIBUTION: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NodeOrigin {
    pub plugin_id: PluginId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contribution_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PluginRenderSlotRequest {
    pub contribution_id: String,
    pub slot_name: String,
    pub route: ResolvedRoute,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub state_snapshot: BTreeMap<String, JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginRenderSlotResponse {
    #[serde(default)]
    pub nodes: Vec<UiNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegisteredSlotContribution {
    pub plugin_id: PluginId,
    pub declaration: SlotContributionDecl,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SlotRegistry {
    by_target: BTreeMap<String, Vec<RegisteredSlotContribution>>,
}

impl SlotRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_plugin(&mut self, manifest: &PluginManifest) -> Result<(), SlotRegistryError> {
        manifest
            .validate_slot_contributions()
            .map_err(SlotRegistryError::InvalidManifest)?;

        self.remove_plugin(&manifest.id);

        for declaration in &manifest.slot_contributions {
            self.by_target
                .entry(declaration.target.clone())
                .or_default()
                .push(RegisteredSlotContribution {
                    plugin_id: manifest.id.clone(),
                    declaration: declaration.clone(),
                });
        }

        self.sort_all();
        Ok(())
    }

    pub fn remove_plugin(&mut self, plugin_id: &PluginId) {
        self.by_target.retain(|_, contributions| {
            contributions.retain(|contribution| &contribution.plugin_id != plugin_id);
            !contributions.is_empty()
        });
    }

    pub fn contributions_for(&self, slot_name: &str) -> &[RegisteredSlotContribution] {
        self.by_target
            .get(slot_name)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn available_for(
        &self,
        slot_name: &str,
        context: &SlotResolutionContext,
    ) -> Vec<RegisteredSlotContribution> {
        let state = MemoryStateStore::new(Some(context.state_snapshot.clone()));
        let mut resolver = DefaultExprResolver::default();
        let ctx = ResolverContext {
            state: &state,
            route: Some(&context.route),
            locale: context.locale.as_deref().unwrap_or_default(),
        };

        self.contributions_for(slot_name)
            .iter()
            .filter(|contribution| {
                contribution
                    .declaration
                    .when
                    .as_ref()
                    .map(|when| resolver.resolve_bool(when, &ctx, None))
                    .unwrap_or(true)
            })
            .cloned()
            .collect()
    }

    pub fn plugin_requires_render_slot_export(manifest: &PluginManifest) -> bool {
        !manifest.slot_contributions.is_empty()
    }

    fn sort_all(&mut self) {
        for contributions in self.by_target.values_mut() {
            contributions.sort_by(|left, right| {
                right
                    .declaration
                    .priority
                    .cmp(&left.declaration.priority)
                    .then_with(|| left.plugin_id.cmp(&right.plugin_id))
                    .then_with(|| left.declaration.id.cmp(&right.declaration.id))
            });
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SlotRegistryError {
    #[error(transparent)]
    InvalidManifest(#[from] crate::core::runtime::ManifestValidationError),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlotResolutionContext {
    pub route: ResolvedRoute,
    pub state_snapshot: BTreeMap<String, JsonValue>,
    pub locale: Option<String>,
    pub limits: SlotResolutionLimits,
}

impl Default for SlotResolutionContext {
    fn default() -> Self {
        Self {
            route: ResolvedRoute::default(),
            state_snapshot: BTreeMap::new(),
            locale: None,
            limits: SlotResolutionLimits::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotResolutionLimits {
    pub max_depth: usize,
    pub max_contributions_per_slot: usize,
    pub max_nodes_per_contribution: usize,
}

impl Default for SlotResolutionLimits {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_SLOT_DEPTH,
            max_contributions_per_slot: DEFAULT_MAX_SLOT_CONTRIBUTIONS_PER_SLOT,
            max_nodes_per_contribution: DEFAULT_MAX_SLOT_NODES_PER_CONTRIBUTION,
        }
    }
}

pub trait SlotContributionRenderer {
    fn render_slot(
        &mut self,
        plugin_id: &PluginId,
        request: &PluginRenderSlotRequest,
    ) -> Result<PluginRenderSlotResponse, SlotRenderError>;
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SlotRenderError {
    #[error("{0}")]
    Message(String),
}

impl From<String> for SlotRenderError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SlotResolutionError {
    #[error("slot resolution cycle detected: {path}")]
    Cycle { path: String },
    #[error("slot resolution exceeded max depth {max_depth}: {path}")]
    MaxDepth { max_depth: usize, path: String },
    #[error("slot `{slot_name}` exceeded max contribution count {max}")]
    TooManyContributions { slot_name: String, max: usize },
    #[error(
        "slot contribution `{plugin_id}:{contribution_id}` returned {count} nodes; max is {max}"
    )]
    TooManyNodes {
        plugin_id: PluginId,
        contribution_id: String,
        count: usize,
        max: usize,
    },
    #[error("slot contribution `{plugin_id}:{contribution_id}` returned invalid AST: {message}")]
    InvalidContributionAst {
        plugin_id: PluginId,
        contribution_id: String,
        message: String,
    },
    #[error("slot contribution `{plugin_id}:{contribution_id}` failed: {message}")]
    RenderContributionFailed {
        plugin_id: PluginId,
        contribution_id: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotResolutionWarning {
    pub plugin_id: PluginId,
    pub contribution_id: String,
    pub slot_name: String,
    pub message: String,
}

#[derive(Debug, Clone)]
struct ResolutionFrame {
    plugin_id: PluginId,
    contribution_id: String,
    slot_name: String,
}

#[derive(Debug, Default)]
struct ResolutionState {
    stack: Vec<ResolutionFrame>,
    warnings: Vec<SlotResolutionWarning>,
}

pub fn resolve_slots(
    mut screen: CanonicalScreen,
    registry: &SlotRegistry,
    context: &SlotResolutionContext,
    render: &mut dyn SlotContributionRenderer,
) -> Result<CanonicalScreen, SlotResolutionError> {
    let mut state = ResolutionState::default();
    screen.children = resolve_children(screen.children, registry, context, render, &mut state)?;
    Ok(screen)
}

pub fn resolve_slots_with_warnings(
    mut screen: CanonicalScreen,
    registry: &SlotRegistry,
    context: &SlotResolutionContext,
    render: &mut dyn SlotContributionRenderer,
) -> Result<(CanonicalScreen, Vec<SlotResolutionWarning>), SlotResolutionError> {
    let mut state = ResolutionState::default();
    screen.children = resolve_children(screen.children, registry, context, render, &mut state)?;
    Ok((screen, state.warnings))
}

pub fn node_origin(node: &CanonicalUiNode) -> Option<&NodeOrigin> {
    node.meta().origin.as_ref()
}

fn resolve_children(
    children: Vec<CanonicalUiNode>,
    registry: &SlotRegistry,
    context: &SlotResolutionContext,
    render: &mut dyn SlotContributionRenderer,
    state: &mut ResolutionState,
) -> Result<Vec<CanonicalUiNode>, SlotResolutionError> {
    let mut out = Vec::new();
    for child in children {
        out.extend(resolve_node(child, registry, context, render, state)?);
    }
    Ok(out)
}

fn resolve_node(
    node: CanonicalUiNode,
    registry: &SlotRegistry,
    context: &SlotResolutionContext,
    render: &mut dyn SlotContributionRenderer,
    state: &mut ResolutionState,
) -> Result<Vec<CanonicalUiNode>, SlotResolutionError> {
    match node {
        CanonicalUiNode::Section(mut node) => {
            node.children = resolve_children(node.children, registry, context, render, state)?;
            Ok(vec![CanonicalUiNode::Section(node)])
        }
        CanonicalUiNode::Stack(mut node) => {
            node.children = resolve_children(node.children, registry, context, render, state)?;
            Ok(vec![CanonicalUiNode::Stack(node)])
        }
        CanonicalUiNode::Inline(mut node) => {
            node.children = resolve_children(node.children, registry, context, render, state)?;
            Ok(vec![CanonicalUiNode::Inline(node)])
        }
        CanonicalUiNode::Grid(mut node) => {
            node.children = resolve_children(node.children, registry, context, render, state)?;
            Ok(vec![CanonicalUiNode::Grid(node)])
        }
        CanonicalUiNode::Scroll(mut node) => {
            node.children = resolve_children(node.children, registry, context, render, state)?;
            Ok(vec![CanonicalUiNode::Scroll(node)])
        }
        CanonicalUiNode::Pressable(mut node) => {
            node.child = Box::new(resolve_single_child(
                *node.child,
                &node.meta.key,
                registry,
                context,
                render,
                state,
            )?);
            Ok(vec![CanonicalUiNode::Pressable(node)])
        }
        CanonicalUiNode::Item(mut node) => {
            node.leading = resolve_children(node.leading, registry, context, render, state)?;
            node.primary = resolve_children(node.primary, registry, context, render, state)?;
            node.secondary = resolve_children(node.secondary, registry, context, render, state)?;
            node.trailing = resolve_children(node.trailing, registry, context, render, state)?;
            Ok(vec![CanonicalUiNode::Item(node)])
        }
        CanonicalUiNode::List(mut node) => {
            node.items = node
                .items
                .into_iter()
                .map(|item| {
                    resolve_node(
                        CanonicalUiNode::Item(item),
                        registry,
                        context,
                        render,
                        state,
                    )
                    .map(|mut nodes| match nodes.remove(0) {
                        CanonicalUiNode::Item(item) => item,
                        _ => unreachable!("item resolver returns item"),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(vec![CanonicalUiNode::List(node)])
        }
        CanonicalUiNode::Actions(mut node) => {
            node.children = node
                .children
                .into_iter()
                .map(|action| {
                    resolve_node(
                        CanonicalUiNode::Action(action),
                        registry,
                        context,
                        render,
                        state,
                    )
                    .map(|mut nodes| match nodes.remove(0) {
                        CanonicalUiNode::Action(action) => action,
                        _ => unreachable!("action resolver returns action"),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(vec![CanonicalUiNode::Actions(node)])
        }
        CanonicalUiNode::Disclosure(mut node) => {
            node.children = resolve_children(node.children, registry, context, render, state)?;
            Ok(vec![CanonicalUiNode::Disclosure(node)])
        }
        CanonicalUiNode::Form(mut node) => {
            node.children = resolve_children(node.children, registry, context, render, state)?;
            Ok(vec![CanonicalUiNode::Form(node)])
        }
        CanonicalUiNode::Status(mut node) => {
            node.actions = node
                .actions
                .into_iter()
                .map(|action| {
                    resolve_node(
                        CanonicalUiNode::Action(action),
                        registry,
                        context,
                        render,
                        state,
                    )
                    .map(|mut nodes| match nodes.remove(0) {
                        CanonicalUiNode::Action(action) => action,
                        _ => unreachable!("action resolver returns action"),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(vec![CanonicalUiNode::Status(node)])
        }
        CanonicalUiNode::Empty(mut node) => {
            node.actions = node
                .actions
                .into_iter()
                .map(|action| {
                    resolve_node(
                        CanonicalUiNode::Action(action),
                        registry,
                        context,
                        render,
                        state,
                    )
                    .map(|mut nodes| match nodes.remove(0) {
                        CanonicalUiNode::Action(action) => action,
                        _ => unreachable!("action resolver returns action"),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(vec![CanonicalUiNode::Empty(node)])
        }
        CanonicalUiNode::Conditional(mut node) => {
            node.r#then = Box::new(resolve_single_child(
                *node.r#then,
                &node.meta.key,
                registry,
                context,
                render,
                state,
            )?);
            node.r#else = match node.r#else {
                Some(child) => Some(Box::new(resolve_single_child(
                    *child,
                    &node.meta.key,
                    registry,
                    context,
                    render,
                    state,
                )?)),
                None => None,
            };
            Ok(vec![CanonicalUiNode::Conditional(node)])
        }
        CanonicalUiNode::Slot(node) => resolve_slot(node, registry, context, render, state),
        leaf => Ok(vec![leaf]),
    }
}

fn resolve_single_child(
    child: CanonicalUiNode,
    parent_key: &str,
    registry: &SlotRegistry,
    context: &SlotResolutionContext,
    render: &mut dyn SlotContributionRenderer,
    state: &mut ResolutionState,
) -> Result<CanonicalUiNode, SlotResolutionError> {
    let mut nodes = resolve_node(child, registry, context, render, state)?;
    Ok(match nodes.len() {
        0 => empty_node(format!("{parent_key}.empty")),
        1 => nodes.remove(0),
        _ => stack_node(format!("{parent_key}.slotGroup"), nodes),
    })
}

fn resolve_slot(
    node: CanonicalSlotNode,
    registry: &SlotRegistry,
    context: &SlotResolutionContext,
    render: &mut dyn SlotContributionRenderer,
    state: &mut ResolutionState,
) -> Result<Vec<CanonicalUiNode>, SlotResolutionError> {
    let contributions = registry.available_for(&node.name, context);
    if contributions.len() > context.limits.max_contributions_per_slot {
        return Err(SlotResolutionError::TooManyContributions {
            slot_name: node.name,
            max: context.limits.max_contributions_per_slot,
        });
    }

    let mut resolved = Vec::new();
    for contribution in contributions {
        detect_cycle_or_depth(&contribution, &node.name, context, state)?;

        let request = PluginRenderSlotRequest {
            contribution_id: contribution.declaration.id.clone(),
            slot_name: node.name.clone(),
            route: context.route.clone(),
            state_snapshot: context.state_snapshot.clone(),
            locale: context.locale.clone(),
        };

        state.stack.push(ResolutionFrame {
            plugin_id: contribution.plugin_id.clone(),
            contribution_id: contribution.declaration.id.clone(),
            slot_name: node.name.clone(),
        });

        let response = match render.render_slot(&contribution.plugin_id, &request) {
            Ok(response) => response,
            Err(error) => {
                let frame = state.stack.pop().expect("pushed slot frame");
                state.warnings.push(SlotResolutionWarning {
                    plugin_id: frame.plugin_id,
                    contribution_id: frame.contribution_id,
                    slot_name: frame.slot_name,
                    message: error.to_string(),
                });
                continue;
            }
        };

        if response.nodes.len() > context.limits.max_nodes_per_contribution {
            state.stack.pop().expect("pushed slot frame");
            return Err(SlotResolutionError::TooManyNodes {
                plugin_id: contribution.plugin_id,
                contribution_id: contribution.declaration.id,
                count: response.nodes.len(),
                max: context.limits.max_nodes_per_contribution,
            });
        }

        let nodes_result = normalize_contribution_nodes(
            response.nodes,
            &contribution.plugin_id,
            &contribution.declaration.id,
        )
        .and_then(|nodes| resolve_children(nodes, registry, context, render, state));
        state.stack.pop().expect("pushed slot frame");
        let nodes = nodes_result?;
        resolved.extend(nodes);
    }

    if resolved.is_empty() {
        match node.fallback {
            Some(fallback) => resolve_node(*fallback, registry, context, render, state),
            None => Ok(vec![]),
        }
    } else {
        Ok(resolved)
    }
}

fn detect_cycle_or_depth(
    contribution: &RegisteredSlotContribution,
    slot_name: &str,
    context: &SlotResolutionContext,
    state: &ResolutionState,
) -> Result<(), SlotResolutionError> {
    if state.stack.len() >= context.limits.max_depth {
        return Err(SlotResolutionError::MaxDepth {
            max_depth: context.limits.max_depth,
            path: format_stack_path(&state.stack),
        });
    }

    if state.stack.iter().any(|frame| {
        frame.plugin_id == contribution.plugin_id
            && frame.contribution_id == contribution.declaration.id
            && frame.slot_name == slot_name
    }) {
        let mut path = state.stack.clone();
        path.push(ResolutionFrame {
            plugin_id: contribution.plugin_id.clone(),
            contribution_id: contribution.declaration.id.clone(),
            slot_name: slot_name.to_string(),
        });
        return Err(SlotResolutionError::Cycle {
            path: format_stack_path(&path),
        });
    }

    Ok(())
}

fn normalize_contribution_nodes(
    mut nodes: Vec<UiNode>,
    plugin_id: &PluginId,
    contribution_id: &str,
) -> Result<Vec<CanonicalUiNode>, SlotResolutionError> {
    for node in &mut nodes {
        namespace_node_ids(node, plugin_id, contribution_id);
    }

    let screen = ScreenNode {
        base: NodeBase {
            id: Some(format!("__slot.{plugin_id}.{contribution_id}.screen",)),
            meta: None,
        },
        title: None,
        subtitle: None,
        initial_focus: None,
        initial_state: None,
        children: nodes,
    };

    let mut screen = normalize_screen(screen).map_err(|message| {
        SlotResolutionError::InvalidContributionAst {
            plugin_id: plugin_id.clone(),
            contribution_id: contribution_id.to_string(),
            message,
        }
    })?;

    let origin = NodeOrigin {
        plugin_id: plugin_id.clone(),
        contribution_id: Some(contribution_id.to_string()),
    };
    for node in &mut screen.children {
        annotate_origin_and_prefix_key(node, &origin, plugin_id, contribution_id);
    }

    Ok(screen.children)
}

fn namespace_node_ids(node: &mut UiNode, plugin_id: &PluginId, contribution_id: &str) {
    let prefix = format!("slot.{plugin_id}.{contribution_id}");
    namespace_node_ids_with_prefix(node, &prefix);
}

fn namespace_id(id: &mut String, prefix: &str) {
    if !id.starts_with(prefix) {
        *id = format!("{prefix}.{id}");
    }
}

fn namespace_base(base: &mut NodeBase, prefix: &str) {
    if let Some(id) = &mut base.id {
        namespace_id(id, prefix);
    }
}

fn namespace_node_ids_with_prefix(node: &mut UiNode, prefix: &str) {
    match node {
        UiNode::Section(node) => {
            namespace_base(&mut node.base, prefix);
            for child in &mut node.children {
                namespace_node_ids_with_prefix(child, prefix);
            }
        }
        UiNode::Stack(node) => {
            namespace_base(&mut node.base, prefix);
            for child in &mut node.children {
                namespace_node_ids_with_prefix(child, prefix);
            }
        }
        UiNode::Inline(node) => {
            namespace_base(&mut node.base, prefix);
            for child in &mut node.children {
                namespace_node_ids_with_prefix(child, prefix);
            }
        }
        UiNode::Grid(node) => {
            namespace_base(&mut node.base, prefix);
            for child in &mut node.children {
                namespace_node_ids_with_prefix(child, prefix);
            }
        }
        UiNode::Scroll(node) => {
            namespace_base(&mut node.base, prefix);
            for child in &mut node.children {
                namespace_node_ids_with_prefix(child, prefix);
            }
        }
        UiNode::Pressable(node) => {
            namespace_base(&mut node.base, prefix);
            namespace_node_ids_with_prefix(&mut node.child, prefix);
        }
        UiNode::Item(node) => {
            namespace_id(&mut node.id, prefix);
            for child in &mut node.leading {
                namespace_node_ids_with_prefix(child, prefix);
            }
            for child in &mut node.primary {
                namespace_node_ids_with_prefix(child, prefix);
            }
            for child in &mut node.secondary {
                namespace_node_ids_with_prefix(child, prefix);
            }
            for child in &mut node.trailing {
                namespace_node_ids_with_prefix(child, prefix);
            }
        }
        UiNode::List(node) => {
            namespace_base(&mut node.base, prefix);
            for item in &mut node.items {
                namespace_id(&mut item.id, prefix);
                for child in &mut item.leading {
                    namespace_node_ids_with_prefix(child, prefix);
                }
                for child in &mut item.primary {
                    namespace_node_ids_with_prefix(child, prefix);
                }
                for child in &mut item.secondary {
                    namespace_node_ids_with_prefix(child, prefix);
                }
                for child in &mut item.trailing {
                    namespace_node_ids_with_prefix(child, prefix);
                }
            }
        }
        UiNode::Actions(node) => {
            namespace_base(&mut node.base, prefix);
            for action in &mut node.children {
                namespace_base(&mut action.base, prefix);
            }
        }
        UiNode::Disclosure(node) => {
            namespace_base(&mut node.base, prefix);
            for child in &mut node.children {
                namespace_node_ids_with_prefix(child, prefix);
            }
        }
        UiNode::Menu(node) => {
            namespace_base(&mut node.base, prefix);
            for item in &mut node.items {
                if let Some(id) = &mut item.id {
                    namespace_id(id, prefix);
                }
            }
        }
        UiNode::Form(node) => {
            namespace_base(&mut node.base, prefix);
            for child in &mut node.children {
                namespace_node_ids_with_prefix(child, prefix);
            }
        }
        UiNode::Status(node) => {
            namespace_base(&mut node.base, prefix);
            for action in &mut node.actions {
                namespace_base(&mut action.base, prefix);
            }
        }
        UiNode::Empty(node) => {
            namespace_base(&mut node.base, prefix);
            for action in &mut node.actions {
                namespace_base(&mut action.base, prefix);
            }
        }
        UiNode::Conditional(node) => {
            namespace_base(&mut node.base, prefix);
            namespace_node_ids_with_prefix(&mut node.r#then, prefix);
            if let Some(child) = &mut node.r#else {
                namespace_node_ids_with_prefix(child, prefix);
            }
        }
        UiNode::Slot(node) => {
            namespace_base(&mut node.base, prefix);
            if let Some(fallback) = &mut node.fallback {
                namespace_node_ids_with_prefix(fallback, prefix);
            }
        }
        UiNode::Text(node) => namespace_base(&mut node.base, prefix),
        UiNode::Value(node) => namespace_base(&mut node.base, prefix),
        UiNode::Icon(node) => namespace_base(&mut node.base, prefix),
        UiNode::Badge(node) => namespace_base(&mut node.base, prefix),
        UiNode::Divider(node) => namespace_base(&mut node.base, prefix),
        UiNode::Media(node) => namespace_base(&mut node.base, prefix),
        UiNode::Action(node) => namespace_base(&mut node.base, prefix),
        UiNode::Input(node) => namespace_base(&mut node.base, prefix),
        UiNode::Loading(node) => namespace_base(&mut node.base, prefix),
    }
}

fn annotate_origin_and_prefix_key(
    node: &mut CanonicalUiNode,
    origin: &NodeOrigin,
    plugin_id: &PluginId,
    contribution_id: &str,
) {
    let prefix = format!("slot.{plugin_id}.{contribution_id}");
    annotate_origin_and_prefix_key_with_prefix(node, origin, &prefix);
}

fn annotate_meta(meta: &mut CanonicalMetadata, origin: &NodeOrigin, prefix: &str) {
    if !meta.key.starts_with(prefix) {
        meta.key = format!("{prefix}.{}", meta.key);
    }
    meta.origin = Some(origin.clone());
}

fn annotate_origin_and_prefix_key_with_prefix(
    node: &mut CanonicalUiNode,
    origin: &NodeOrigin,
    prefix: &str,
) {
    match node {
        CanonicalUiNode::Section(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            for child in &mut node.children {
                annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
            }
        }
        CanonicalUiNode::Stack(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            for child in &mut node.children {
                annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
            }
        }
        CanonicalUiNode::Inline(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            for child in &mut node.children {
                annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
            }
        }
        CanonicalUiNode::Grid(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            for child in &mut node.children {
                annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
            }
        }
        CanonicalUiNode::Scroll(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            for child in &mut node.children {
                annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
            }
        }
        CanonicalUiNode::Pressable(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            annotate_origin_and_prefix_key_with_prefix(&mut node.child, origin, prefix);
        }
        CanonicalUiNode::Item(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            for child in &mut node.leading {
                annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
            }
            for child in &mut node.primary {
                annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
            }
            for child in &mut node.secondary {
                annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
            }
            for child in &mut node.trailing {
                annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
            }
        }
        CanonicalUiNode::List(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            for item in &mut node.items {
                annotate_meta(&mut item.meta, origin, prefix);
                for child in &mut item.leading {
                    annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
                }
                for child in &mut item.primary {
                    annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
                }
                for child in &mut item.secondary {
                    annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
                }
                for child in &mut item.trailing {
                    annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
                }
            }
        }
        CanonicalUiNode::Actions(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            for action in &mut node.children {
                annotate_meta(&mut action.meta, origin, prefix);
            }
        }
        CanonicalUiNode::Disclosure(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            for child in &mut node.children {
                annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
            }
        }
        CanonicalUiNode::Form(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            for child in &mut node.children {
                annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
            }
        }
        CanonicalUiNode::Status(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            for action in &mut node.actions {
                annotate_meta(&mut action.meta, origin, prefix);
            }
        }
        CanonicalUiNode::Empty(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            for action in &mut node.actions {
                annotate_meta(&mut action.meta, origin, prefix);
            }
        }
        CanonicalUiNode::Conditional(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            annotate_origin_and_prefix_key_with_prefix(&mut node.r#then, origin, prefix);
            if let Some(child) = &mut node.r#else {
                annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
            }
        }
        CanonicalUiNode::Slot(node) => {
            annotate_meta(&mut node.meta, origin, prefix);
            if let Some(child) = &mut node.fallback {
                annotate_origin_and_prefix_key_with_prefix(child, origin, prefix);
            }
        }
        CanonicalUiNode::Text(node) => annotate_meta(&mut node.meta, origin, prefix),
        CanonicalUiNode::Value(node) => annotate_meta(&mut node.meta, origin, prefix),
        CanonicalUiNode::Icon(node) => annotate_meta(&mut node.meta, origin, prefix),
        CanonicalUiNode::Badge(node) => annotate_meta(&mut node.meta, origin, prefix),
        CanonicalUiNode::Divider(node) => annotate_meta(&mut node.meta, origin, prefix),
        CanonicalUiNode::Media(node) => annotate_meta(&mut node.meta, origin, prefix),
        CanonicalUiNode::Action(node) => annotate_meta(&mut node.meta, origin, prefix),
        CanonicalUiNode::Menu(node) => annotate_meta(&mut node.meta, origin, prefix),
        CanonicalUiNode::Input(node) => annotate_meta(&mut node.meta, origin, prefix),
        CanonicalUiNode::Loading(node) => annotate_meta(&mut node.meta, origin, prefix),
    }
}

fn empty_node(key: String) -> CanonicalUiNode {
    CanonicalUiNode::Empty(CanonicalEmptyStateNode {
        base: NodeBase::default(),
        icon: None,
        title: OneOrExpr::Value(String::new()),
        message: None,
        actions: vec![],
        meta: static_meta(key),
    })
}

fn stack_node(key: String, children: Vec<CanonicalUiNode>) -> CanonicalUiNode {
    CanonicalUiNode::Stack(CanonicalStackNode {
        base: NodeBase::default(),
        gap: None,
        children,
        meta: static_meta(key),
    })
}

fn static_meta(key: String) -> CanonicalMetadata {
    CanonicalMetadata {
        key,
        reactivity: NodeReactivity::Static,
        subtree_reactivity: NodeReactivity::Static,
        reactive_fields: vec![],
        shape_reactivity: ShapeReactivity::Static,
        subtree_shape_reactivity: ShapeReactivity::Static,
        structural_dependencies: vec![],
        static_fields: BTreeMap::new(),
        origin: None,
    }
}

fn format_stack_path(stack: &[ResolutionFrame]) -> String {
    stack
        .iter()
        .map(|frame| {
            format!(
                "{}:{}@{}",
                frame.plugin_id, frame.contribution_id, frame.slot_name
            )
        })
        .collect::<Vec<_>>()
        .join(" -> ")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::*;
    use crate::core::ast::{ActionRef, ActionType, StatusSeverity, UiExpr};
    use crate::core::dsl::{self as ui, IntoNode};
    use crate::core::ir::lower_screen;
    use crate::core::normalize::normalize_screen;
    use crate::core::permissions::{PermissionGrant, PermissionGuard, PermissionProfile};
    use crate::core::runtime::{PluginManifest, SlotContributionDecl};

    #[derive(Debug, Default)]
    struct FakeRenderer {
        responses: BTreeMap<(PluginId, String), Result<PluginRenderSlotResponse, SlotRenderError>>,
        calls: Vec<(PluginId, PluginRenderSlotRequest)>,
    }

    impl FakeRenderer {
        fn respond(
            mut self,
            plugin_id: impl Into<String>,
            contribution_id: impl Into<String>,
            nodes: Vec<UiNode>,
        ) -> Self {
            self.responses.insert(
                (plugin_id.into(), contribution_id.into()),
                Ok(PluginRenderSlotResponse { nodes }),
            );
            self
        }

        fn fail(
            mut self,
            plugin_id: impl Into<String>,
            contribution_id: impl Into<String>,
            message: impl Into<String>,
        ) -> Self {
            self.responses.insert(
                (plugin_id.into(), contribution_id.into()),
                Err(SlotRenderError::Message(message.into())),
            );
            self
        }
    }

    impl SlotContributionRenderer for FakeRenderer {
        fn render_slot(
            &mut self,
            plugin_id: &PluginId,
            request: &PluginRenderSlotRequest,
        ) -> Result<PluginRenderSlotResponse, SlotRenderError> {
            self.calls.push((plugin_id.clone(), request.clone()));
            self.responses
                .get(&(plugin_id.clone(), request.contribution_id.clone()))
                .cloned()
                .unwrap_or_else(|| Ok(PluginRenderSlotResponse::default()))
        }
    }

    fn manifest(plugin_id: &str, contributions: Vec<SlotContributionDecl>) -> PluginManifest {
        PluginManifest {
            id: plugin_id.to_string(),
            name: plugin_id.to_string(),
            slot_contributions: contributions,
            ..PluginManifest::default()
        }
    }

    fn contribution(id: &str, target: &str, priority: i32) -> SlotContributionDecl {
        SlotContributionDecl {
            id: id.to_string(),
            target: target.to_string(),
            priority,
            when: None,
        }
    }

    fn screen_with(node: impl IntoNode) -> CanonicalScreen {
        normalize_screen(ui::screen().child(node).build()).expect("screen")
    }

    #[test]
    fn slot_without_contributions_uses_fallback() {
        let screen = screen_with(
            ui::slot("catalog.work-detail:footer")
                .id("slot.footer")
                .fallback(ui::text("Fallback")),
        );
        let mut renderer = FakeRenderer::default();

        let resolved = resolve_slots(
            screen,
            &SlotRegistry::default(),
            &SlotResolutionContext::default(),
            &mut renderer,
        )
        .expect("resolved");

        assert_eq!(resolved.children.len(), 1);
        assert!(matches!(resolved.children[0], CanonicalUiNode::Text(_)));
        assert!(renderer.calls.is_empty());
    }

    #[test]
    fn slot_without_contributions_and_fallback_is_removed() {
        let screen = screen_with(ui::slot("catalog.work-detail:footer").id("slot.footer"));
        let mut renderer = FakeRenderer::default();

        let resolved = resolve_slots(
            screen,
            &SlotRegistry::default(),
            &SlotResolutionContext::default(),
            &mut renderer,
        )
        .expect("resolved");

        assert!(resolved.children.is_empty());
    }

    #[test]
    fn resolves_contribution_and_preserves_origin() {
        let mut registry = SlotRegistry::new();
        registry
            .register_plugin(&manifest(
                "reviews.plugin",
                vec![contribution(
                    "reviews-summary",
                    "catalog.work-detail:footer",
                    10,
                )],
            ))
            .expect("registered");
        let screen = screen_with(
            ui::slot("catalog.work-detail:footer")
                .id("slot.footer")
                .fallback(ui::text("Fallback")),
        );
        let action = ActionRef {
            r#type: ActionType::Custom("reviews.open".to_string()),
            params: None,
            confirm: None,
        };
        let mut renderer = FakeRenderer::default().respond(
            "reviews.plugin",
            "reviews-summary",
            vec![ui::action("Reviews", action).id("open").into_node()],
        );

        let resolved = resolve_slots(
            screen,
            &registry,
            &SlotResolutionContext::default(),
            &mut renderer,
        )
        .expect("resolved");

        assert_eq!(resolved.children.len(), 1);
        let origin = node_origin(&resolved.children[0]).expect("origin");
        assert_eq!(origin.plugin_id, "reviews.plugin");
        assert_eq!(origin.contribution_id.as_deref(), Some("reviews-summary"));
        assert!(resolved.children[0].meta().key.contains("reviews.plugin"));
        assert_eq!(renderer.calls.len(), 1);
        assert_eq!(renderer.calls[0].0, "reviews.plugin");
        assert_eq!(renderer.calls[0].1.contribution_id, "reviews-summary");

        let ir = lower_screen(&resolved);
        assert_eq!(ir.c[0].p[SLOT_ORIGIN_PLUGIN_ID], "reviews.plugin");
        assert_eq!(ir.c[0].p[SLOT_ORIGIN_CONTRIBUTION_ID], "reviews-summary");
    }

    #[test]
    fn nested_contributed_actions_carry_contributor_origin() {
        let mut registry = SlotRegistry::new();
        registry
            .register_plugin(&manifest(
                "reviews.plugin",
                vec![contribution("actions", "slot.target", 0)],
            ))
            .expect("registered");
        let action = || ActionRef {
            r#type: ActionType::Custom("reviews.open".to_string()),
            params: None,
            confirm: None,
        };
        let screen = screen_with(ui::slot("slot.target").id("slot.target.node"));
        let mut renderer = FakeRenderer::default().respond(
            "reviews.plugin",
            "actions",
            vec![
                ui::actions()
                    .id("group")
                    .child(ui::action("Grouped", action()).id("grouped"))
                    .into_node(),
                ui::status(StatusSeverity::Info, "Status")
                    .id("status")
                    .action(ui::action("Status action", action()).id("status.action"))
                    .into_node(),
                ui::empty("Empty")
                    .id("empty")
                    .action(ui::action("Empty action", action()).id("empty.action"))
                    .into_node(),
            ],
        );

        let resolved = resolve_slots(
            screen,
            &registry,
            &SlotResolutionContext::default(),
            &mut renderer,
        )
        .expect("resolved");

        let ir = lower_screen(&resolved);
        let mut action_origins = Vec::new();
        collect_ir_action_origins(&ir.c, &mut action_origins);

        assert_eq!(action_origins.len(), 3);
        assert!(
            action_origins
                .iter()
                .all(|origin| origin == "reviews.plugin")
        );
    }

    #[test]
    fn contributed_action_permissions_are_selected_from_origin_plugin() {
        let mut registry = SlotRegistry::new();
        registry
            .register_plugin(&manifest(
                "reviews.plugin",
                vec![contribution("reviews-summary", "slot.target", 0)],
            ))
            .expect("registered");
        let screen = screen_with(ui::slot("slot.target").id("slot.target.node"));
        let mut renderer = FakeRenderer::default().respond(
            "reviews.plugin",
            "reviews-summary",
            vec![
                ui::action(
                    "Moderate reviews",
                    ActionRef {
                        r#type: ActionType::Custom("reviews.moderate".to_string()),
                        params: None,
                        confirm: None,
                    },
                )
                .id("moderate")
                .into_node(),
            ],
        );
        let resolved = resolve_slots(
            screen,
            &registry,
            &SlotResolutionContext::default(),
            &mut renderer,
        )
        .expect("resolved");
        let action_origin = node_origin(&resolved.children[0]).expect("origin");
        let guards = BTreeMap::from([
            ("screen.plugin".to_string(), guard("screen.plugin", vec![])),
            (
                "reviews.plugin".to_string(),
                guard("reviews.plugin", vec!["reviews.write"]),
            ),
        ]);

        assert!(guards["screen.plugin"].assert("reviews.write").is_err());
        assert!(
            guards[&action_origin.plugin_id]
                .assert("reviews.write")
                .is_ok()
        );
    }

    #[test]
    fn orders_by_priority_then_plugin_and_contribution_id() {
        let mut registry = SlotRegistry::new();
        registry
            .register_plugin(&manifest(
                "z.plugin",
                vec![contribution("b", "slot.target", 10)],
            ))
            .expect("registered");
        registry
            .register_plugin(&manifest(
                "a.plugin",
                vec![
                    contribution("b", "slot.target", 10),
                    contribution("a", "slot.target", 100),
                ],
            ))
            .expect("registered");

        let ordered = registry
            .contributions_for("slot.target")
            .iter()
            .map(|entry| format!("{}:{}", entry.plugin_id, entry.declaration.id))
            .collect::<Vec<_>>();

        assert_eq!(ordered, vec!["a.plugin:a", "a.plugin:b", "z.plugin:b"]);
    }

    #[test]
    fn filters_when_expression_against_state() {
        let mut registry = SlotRegistry::new();
        let mut hidden = contribution("hidden", "slot.target", 0);
        hidden.when = Some(OneOrExpr::Expr(UiExpr::Binding {
            path: "feature.hidden".to_string(),
        }));
        let mut visible = contribution("visible", "slot.target", 0);
        visible.when = Some(OneOrExpr::Expr(UiExpr::Binding {
            path: "feature.visible".to_string(),
        }));
        registry
            .register_plugin(&manifest("plugin", vec![hidden, visible]))
            .expect("registered");

        let context = SlotResolutionContext {
            state_snapshot: BTreeMap::from([("feature.visible".to_string(), json!(true))]),
            ..SlotResolutionContext::default()
        };
        let available = registry.available_for("slot.target", &context);

        assert_eq!(available.len(), 1);
        assert_eq!(available[0].declaration.id, "visible");
    }

    #[test]
    fn empty_contribution_falls_back_when_nothing_valid_remains() {
        let mut registry = SlotRegistry::new();
        registry
            .register_plugin(&manifest(
                "reviews.plugin",
                vec![contribution("empty", "slot.target", 0)],
            ))
            .expect("registered");
        let screen = screen_with(
            ui::slot("slot.target")
                .id("slot.target.node")
                .fallback(ui::text("Fallback")),
        );
        let mut renderer = FakeRenderer::default().respond("reviews.plugin", "empty", vec![]);

        let resolved = resolve_slots(
            screen,
            &registry,
            &SlotResolutionContext::default(),
            &mut renderer,
        )
        .expect("resolved");

        assert!(matches!(resolved.children[0], CanonicalUiNode::Text(_)));
    }

    #[test]
    fn failed_contributor_does_not_block_other_contributors() {
        let mut registry = SlotRegistry::new();
        registry
            .register_plugin(&manifest(
                "bad.plugin",
                vec![contribution("broken", "slot.target", 100)],
            ))
            .expect("registered");
        registry
            .register_plugin(&manifest(
                "good.plugin",
                vec![contribution("ok", "slot.target", 0)],
            ))
            .expect("registered");
        let screen = screen_with(ui::slot("slot.target").id("slot.target.node"));
        let mut renderer = FakeRenderer::default()
            .fail("bad.plugin", "broken", "boom")
            .respond("good.plugin", "ok", vec![ui::text("OK").into_node()]);

        let (resolved, warnings) = resolve_slots_with_warnings(
            screen,
            &registry,
            &SlotResolutionContext::default(),
            &mut renderer,
        )
        .expect("resolved");

        assert_eq!(resolved.children.len(), 1);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].plugin_id, "bad.plugin");
    }

    #[test]
    fn detects_direct_cycle() {
        let mut registry = SlotRegistry::new();
        registry
            .register_plugin(&manifest(
                "loop.plugin",
                vec![contribution("again", "slot.target", 0)],
            ))
            .expect("registered");
        let screen = screen_with(ui::slot("slot.target").id("slot.target.node"));
        let mut renderer = FakeRenderer::default().respond(
            "loop.plugin",
            "again",
            vec![ui::slot("slot.target").id("nested.slot").into_node()],
        );

        let err = resolve_slots(
            screen,
            &registry,
            &SlotResolutionContext::default(),
            &mut renderer,
        )
        .expect_err("cycle");

        assert!(matches!(err, SlotResolutionError::Cycle { .. }));
    }

    #[test]
    fn validates_manifest_contributions() {
        let manifest = manifest(
            "plugin",
            vec![
                contribution("duplicate", "slot.target", 0),
                contribution("duplicate", "slot.target", 1),
            ],
        );

        assert!(manifest.validate_slot_contributions().is_err());
    }

    #[test]
    fn serializes_render_slot_envelope() {
        let request = PluginRenderSlotRequest {
            contribution_id: "reviews-summary".to_string(),
            slot_name: "catalog.work-detail:footer".to_string(),
            route: ResolvedRoute {
                pattern: "/works/:id".to_string(),
                params: BTreeMap::from([("id".to_string(), "42".to_string())]),
                query: BTreeMap::new(),
            },
            state_snapshot: BTreeMap::from([("ui.open".to_string(), json!(true))]),
            locale: Some("en".to_string()),
        };

        let encoded = serde_json::to_string(&request).expect("json");
        let decoded: PluginRenderSlotRequest = serde_json::from_str(&encoded).expect("decoded");

        assert_eq!(decoded, request);
    }

    fn guard(plugin_id: &str, permissions: Vec<&str>) -> PermissionGuard {
        PermissionGuard::new(PermissionProfile {
            plugin_id: plugin_id.to_string(),
            grants: permissions
                .into_iter()
                .map(|permission| PermissionGrant {
                    permission: permission.to_string(),
                    granted: true,
                    granted_at: "2026-07-16T00:00:00Z".to_string(),
                    allowed_origins: vec![],
                })
                .collect(),
        })
    }

    fn collect_ir_action_origins(nodes: &[crate::core::ir::IrNode], out: &mut Vec<String>) {
        for node in nodes {
            if node.t == "action" {
                out.push(
                    node.p
                        .get(SLOT_ORIGIN_PLUGIN_ID)
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                );
            }
            collect_ir_action_origins(&node.c, out);
        }
    }
}
