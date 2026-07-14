# UNode DSL em Rust

DSL idiomática em Rust para construir a AST serializável do uNode.

A ideia central é simples:

- a **AST** continua sendo o contrato canônico, estável e serializável;
- a **DSL** é só uma camada ergonômica para autoria;
- a sintaxe tenta seguir o **Rust normal**, sem inventar uma mini linguagem paralela;
- macros são usadas só onde realmente compensam.

---

## Objetivos da DSL

Esta DSL foi desenhada para resolver alguns problemas comuns ao portar uma API declarativa de UI de TypeScript para Rust:

- composição de subárvores sem verbosidade excessiva;
- suporte natural a conteúdo opcional;
- suporte a listas e iteradores;
- distinção clara entre:
  - condição resolvida no host/plugin;
  - condição simbólica que precisa ir para a AST e ser resolvida pelo renderer.

---

## Princípios de design

### 1. Separação entre AST e DSL

A AST representa o protocolo real da UI.

A DSL não redefine a AST nem tenta escondê-la; ela apenas facilita sua construção.

Isso permite:

- serializar a AST sem depender da DSL;
- trocar a DSL no futuro sem quebrar o protocolo;
- gerar AST por builders, macros, parser, plugin wasm ou qualquer outro caminho.

### 2. Builders fluentes

Cada nó importante possui um builder fluente, por exemplo:

- `screen()`
- `stack()`
- `section()`
- `inline()`
- `grid()`
- `text()`
- `value()`
- `status()`
- `empty()`
- `pressable()`
- `action()`
- `form()`

### 3. Traits de composição

A ergonomia da DSL depende principalmente destes traits:

- `IntoNode`
- `IntoChildren`
- `IntoAction`
- `IntoActions`
- `IntoItem`
- `IntoItems`
- `IntoMenuItem`
- `IntoMenuItems`

Eles permitem aceitar builders, nós já prontos, `Option`, arrays, `Vec` e wrappers de iteradores com uma API uniforme.

### 4. Macro mínima

A macro `ui_children![]` existe apenas para composição e flattening de filhos.

Ela não tenta virar uma linguagem própria de template.

---

## Conceitos principais

### `IntoNode`

Qualquer coisa que possa virar um `UiNode`.

Exemplos:

- um `UiNode` já pronto;
- um builder como `TextBuilder`, `StackBuilder`, `GridBuilder`, etc.;
- alguns nós canônicos da AST.

Uso prático:

```rust
stack().child(text("Olá"))
pressable(text("Abrir"), action_ref)
```

Nesses casos, o builder é aceito porque implementa `IntoNode`.

---

### `IntoChildren`

Qualquer coisa que possa virar `Vec<UiNode>`.

Isso inclui:

- um único nó;
- `Option<T>`;
- `[T; N]`;
- `Vec<T>`;
- `Children`, que é um wrapper para iteradores.

Exemplo:

```rust
stack()
    .children([
        text("Título"),
        text("Subtítulo"),
    ])
```

Também funciona com opcionais:

```rust
stack().children(when(show_error, status(Tone::Warning, "Falhou")))
```

---

### `children(iter)`

Quando você tem um iterador, usa o helper `children(...)`.

Exemplo:

```rust
grid().children(children(
    banners.into_iter().map(|banner| {
        pressable(
            text(banner.title),
            banner.action,
        )
    })
))
```

Isso evita ter que fazer `collect::<Vec<_>>()` toda hora.

---

## Expressões

A AST suporta valores literais e expressões simbólicas.

A DSL expõe helpers no módulo `expr`:

```rust
expr::literal(...)
expr::binding(...)
expr::param(...)
```

### Exemplo

```rust
text("Título fixo")
text(expr::binding("state.title"))
text(expr::param("workId"))
```

### Interpretação

- `"Título fixo"` vira valor direto;
- `expr::binding("state.title")` representa binding resolvido pelo runtime;
- `expr::param("workId")` representa um parâmetro de rota ou contexto.

---

## Condicionais

A DSL separa claramente dois casos.

### 1. Condição resolvida no host/plugin

Use:

- `when(cond, node)`
- `when_some(opt, |v| ...)`

Esses helpers **não geram `ConditionalNode`**.
Eles apenas montam ou omitem subárvore durante a construção.

### Exemplo

```rust
stack().children(ui_children![
    text("Catálogo"),
    when_some(input.error.clone(), |err| {
        status(Tone::Warning, err)
            .title("Falha ao carregar")
    }),
])
```

---

### 2. Condição resolvida pelo renderer

Use `conditional(...)`.

Isso **gera um `ConditionalNode` real na AST**.

### Exemplo

```rust
conditional(
    expr::binding("state.has_error"),
    status(Tone::Warning, "Falha"),
    Some(empty("Sem conteúdo")),
)
```

Esse caso é útil quando a decisão depende de estado dinâmico já no ambiente de execução da UI.

---

# Uso da nova DSL

Abaixo estão exemplos representativos de como fica o uso com a versão revisada da DSL.

---

## Exemplo simples: texto em stack

```rust
stack()
    .gap(Gap::Md)
    .children([
        text("Título").role(TextRole::Heading),
        text("Subtítulo").role(TextRole::Subtitle),
    ])
```

---

## Exemplo com conteúdo opcional

```rust
stack()
    .gap(Gap::Lg)
    .children(ui_children![
        text("Resultados").role(TextRole::Heading),
        when_some(error.clone(), |err| {
            status(Tone::Warning, err)
                .title("Erro")
        }),
    ])
```

---

## Exemplo com grid e iterador

```rust
grid()
    .columns(cols().base(1).sm(2).md(3).lg(4))
    .gap(Gap::Lg)
    .children(children(
        items.into_iter().map(|item| {
            pressable(
                text(item.title),
                item.action,
            )
            .label(item.title)
        })
    ))
```

---

## Exemplo com lista semântica

```rust
list([
    item("work-1", text("Frieren"))
        .secondary(text("Fantasy"))
        .action(ActionRef {
            r#type: "open_work".into(),
            payload: None,
        }),
    item("work-2", text("Dungeon Meshi"))
        .secondary(text("Adventure"))
        .action(ActionRef {
            r#type: "open_work".into(),
            payload: None,
        }),
])
```

---

## Exemplo com actions

A versão revisada da DSL deixa `action(...)` e `actions()` mais consistentes.

```rust
actions()
    .align(Align::End)
    .children([
        action("Cancelar", ActionRef {
            r#type: "cancel".into(),
            payload: None,
        }),
        action("Salvar", ActionRef {
            r#type: "save".into(),
            payload: None,
        })
        .intent(ActionIntent::Primary),
    ])
```

Você também pode adicionar uma por uma:

```rust
actions()
    .child(
        action("Voltar", ActionRef {
            r#type: "back".into(),
            payload: None,
        })
    )
    .child(
        action("Continuar", ActionRef {
            r#type: "continue".into(),
            payload: None,
        })
        .intent(ActionIntent::Primary)
    )
```

---

## Exemplo com formulário

```rust
form("contact")
    .submit(ActionRef {
        r#type: "submit_contact".into(),
        payload: None,
    })
    .children([
        text("Contato").role(TextRole::Heading),

        input("name", InputKind::Text, "Nome")
            .required(true)
            .placeholder("Seu nome completo"),

        input("email", InputKind::Email, "E-mail")
            .required(true)
            .placeholder("voce@exemplo.com"),

        input("message", InputKind::Textarea, "Mensagem")
            .help_text("Descreva sua dúvida"),

        actions()
            .align(Align::End)
            .child(
                action("Enviar", ActionRef {
                    r#type: "submit_contact".into(),
                    payload: None,
                })
                .intent(ActionIntent::Primary)
            ),
    ])
```

---

## Exemplo com `value(...)` usando literal e binding

Uma das correções da DSL revisada é permitir que `value(...)` aceite tanto valor primitivo quanto expressão.

### Literal

```rust
value(42, ValueFormat::Number)
```

### Binding

```rust
value(expr::binding("state.price"), ValueFormat::Currency)
    .currency_code("BRL")
```

Isso deixa `ValueNode` coerente com a própria AST, que já suporta `PrimitiveOrExpr`.

---

## Exemplo realista: tela de catálogo

Abaixo, um exemplo mais próximo do seu caso de uso original.

```rust
screen()
    .id("catalog")
    .title("Browse Catalog")
    .child(
        stack()
            .gap(Gap::Lg)
            .children(ui_children![
                stack()
                    .gap(Gap::Xs)
                    .children([
                        text("Browse Catalog").role(TextRole::Heading),
                        text("Discover works and updates").role(TextRole::Subtitle),
                    ]),

                when_some(input.error.clone(), |err| {
                    status(Tone::Warning, err)
                        .title("Failed to load")
                }),

                if banners.is_empty() {
                    empty("Browse Catalog")
                        .message(input.empty_text.clone())
                        .into_node()
                } else {
                    grid()
                        .columns(cols().base(1).sm(2).md(3).lg(4).xl(5))
                        .gap(Gap::Lg)
                        .children(children(
                            banners.into_iter().map(|banner| {
                                pressable(
                                    work_banner(banner.view_model.clone()),
                                    banner.action.clone(),
                                )
                                .label(banner.view_model.title.clone())
                            })
                        ))
                        .into_node()
                }
            ])
    )
    .build()
```

---

# Como pensar a DSL no dia a dia

A forma mais saudável de usar essa DSL é esta:

- use builders normalmente;
- use `children([...])` ou arrays quando a estrutura for simples;
- use `ui_children![]` quando houver composição mista;
- use `children(iter)` quando vier de iterador;
- use `when(...)` e `when_some(...)` para condicionais locais;
- use `conditional(...)` só quando a condição realmente precisar existir na AST.

---

# Macro `ui_children!`

A macro continua propositalmente pequena.

Exemplo:

```rust
ui_children![
    text("A"),
    text("B"),
    when(show_extra, text("C")),
    when_some(optional_label, |label| text(label)),
    children(items.into_iter().map(|item| text(item.title))),
]
```

A função dela é só montar um `Vec<UiNode>` a partir de expressões que já implementam `IntoChildren`.

Ela não tenta:

- interpretar `for`;
- interpretar `match`;
- criar sintaxe customizada;
- competir com o parser do Rust.

Isso é uma escolha intencional.

---

# Diferenças em relação à versão TypeScript

### 1. Builders no lugar de objetos inline

Em TypeScript, era natural escrever algo como:

```ts
ui.stack({ gap: 'lg' }, [...])
```

Em Rust, o mais idiomático é:

```rust
stack().gap(Gap::Lg).children([...])
```

### 2. Flattening explícito

Em TypeScript é comum espalhar arrays com `...`.

Em Rust, isso fica melhor com:

- `IntoChildren`
- `children(iter)`
- `ui_children![]`

### 3. Condicionais mais bem separadas

Na versão revisada da DSL, a diferença entre:

- montar/omitir nó localmente;
- gerar um nó condicional real na AST

fica muito mais explícita.

### 4. Iteradores como caminho principal para coleções dinâmicas

Em Rust, é mais natural aproveitar `into_iter().map(...)` do que simular arrays temporários como no TS.

---

# Problemas da AST que ainda precisam ser melhorados

A DSL ficou melhor, mas existem limitações que vêm da própria AST atual.

Esses pontos não são "problemas da DSL"; são limites do modelo de dados.

### 1. `TextNode.emphasis` ainda é `Option<String>`

Esse é o exemplo mais evidente.

Hoje, `TextBuilder.emphasis(...)` continua aceitando string porque a AST ainda define esse campo assim.

Isso é fraco para Rust por dois motivos:

- perde validação estática;
- permite valores arbitrários fora do domínio esperado.

O ideal aqui seria trocar para um enum, algo como:

```rust
pub enum TextEmphasis {
    Normal,
    Strong,
}
```

ou, se quiser mais flexibilidade:

```rust
pub enum TextEmphasis {
    Normal,
    Medium,
    Strong,
}
```

Enquanto isso não for corrigido na AST, a DSL inevitavelmente continua "stringly typed" nesse ponto.

---

### 2. `StatusNode.severity` usa `Tone`

Hoje `status(...)` reutiliza `Tone` como severidade.

Funciona, mas semanticamente não é o melhor tipo.

`Tone` parece representar estilo visual geral, enquanto severidade representa intenção de feedback, por exemplo:

- info
- success
- warning
- danger

O ideal seria uma enum própria, como:

```rust
pub enum StatusSeverity {
    Info,
    Success,
    Warning,
    Danger,
}
```

Isso deixa a AST mais semântica e evita reaproveitamento excessivo de tipos.

---

### 3. `Align` está permissivo demais em alguns contextos

Há lugares em que o mesmo tipo `Align` é reaproveitado, mas nem todos os valores parecem válidos semanticamente.

Se um nó aceita só parte do domínio, isso é um sinal de que talvez o tipo esteja genérico demais.

Duas saídas possíveis:

- criar enums menores e específicas por contexto;
- ou manter a enum ampla, mas validar isso explicitamente em outra camada.

Minha preferência é tipar melhor.

---

### 4. Alguns campos ainda parecem "stringly typed" demais

Além de `TextNode.emphasis`, a AST ainda tem outros pontos onde strings livres parecem representar domínios fechados.

Esse tipo de coisa em Rust costuma envelhecer mal.

Sempre que um campo representa um conjunto fechado de opções, vale preferir enum.

---

### 5. `density` em `ListNode` como string é fraco

Se o domínio esperado for algo como:

- compact
- normal
- comfortable

então isso deveria ser uma enum.

Por exemplo:

```rust
pub enum ListDensity {
    Compact,
    Normal,
    Comfortable,
}
```

Isso melhora:

- segurança de tipo;
- autocomplete;
- serialização previsível;
- qualidade da DSL.

---

### 6. `media_kind: String` pode estar aberto demais

Talvez isso seja proposital, mas vale revisar.

Se `media_kind` representa categorias conhecidas como:

- image
- video
- audio
- cover

talvez uma enum faça mais sentido.

Se ele realmente precisa ser extensível por terceiros, então a string pode continuar válida. Aqui depende da ambição do protocolo.

---

### 7. `ActionRef.type` como string pode merecer revisão

Se a ideia é suportar ações abertas e extensíveis, string faz sentido.

Mas se o protocolo tiver um núcleo fixo forte, talvez valha modelar algo híbrido:

```rust
pub enum ActionType {
    Core(CoreActionType),
    Custom(String),
}
```

Isso dá um meio termo bom entre segurança e extensibilidade.

---

### 8. Falta revisar onde a AST quer ser "semântica" e onde quer ser "genérica"

Esse é o ponto mais importante no longo prazo.

A AST parece oscilar entre dois estilos:

- **semântico**, com nós como `StatusNode`, `EmptyStateNode`, `DisclosureNode`, `PressableNode`;
- **genérico**, com vários campos livres por string.

Os dois estilos podem coexistir, mas precisam de um critério claro.

Minha recomendação é:

- ser mais fechado e tipado no que é núcleo do protocolo;
- deixar aberto por string só onde você realmente quer extensão por plugins/hosts.

---

# Conclusão

A nova DSL já está num ponto bom:

- builders fluentes;
- composição previsível;
- condicionais bem separadas;
- iteradores bem suportados;
- macro mínima e saudável.

O próximo salto de qualidade não vem de mais açúcar sintático.

Vem de **endurecer a AST**, trocando campos frouxos por tipos mais semânticos e mais seguros.

Se eu fosse priorizar as próximas melhorias, eu atacaria nesta ordem:

1. `TextNode.emphasis` virar enum;
2. `ListNode.density` virar enum;
3. `StatusNode.severity` ganhar enum própria;
4. revisar campos stringly typed restantes;
5. decidir melhor onde o protocolo quer ser fechado e onde quer ser extensível.