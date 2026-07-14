# Status Da Migracao TypeScript -> Rust + Wasm

Data deste snapshot: 2026-04-03.

## Objetivo

O objetivo desta migracao nao e apenas portar o que existe hoje em TypeScript para Rust.

Queremos aproveitar a troca de stack para:

- endurecer o protocolo da `unode`;
- melhorar sandbox e boundary de execucao via WASM;
- separar melhor o que e:
  - core generico da `unode`;
  - SDK de autoria de plugin (`unode-sdk`);
  - dominio Mugens e DTOs (`mugens-domain`);
  - bridge/APIs de dominio para plugins (`mugens-sdk`);
  - renderer e infraestrutura de execucao (`renderer`, `unode-web-runtime`);
- remover problemas conhecidos da implementacao TypeScript que hoje estao misturando renderer, runtime, host app e dominio.

Este documento resume o estado real do repositorio hoje, o que ja foi portado, o que ainda falta e quais decisoes arquiteturais precisam ser travadas antes de acelerar implementacao.

## Snapshot Rapido

Hoje o estado do repositorio e o seguinte:

- `ts-implementation/` ainda e a fonte de verdade funcional da arquitetura.
- `crates/unode/src/core` ja contem uma base Rust relevante.
- o resto das crates Rust ainda esta praticamente em placeholder.
- o workspace Rust compila, mas hoje tem zero testes.

Em termos praticos:

- o sistema antigo em TypeScript ainda contem runtime, registries, loader, state, resolver, i18n, permissao e renderer;
- o sistema novo em Rust hoje contem AST, DSL, normalize e uma camada de lowering/transport;
- ainda nao existe pipeline ponta a ponta em Rust para plugin -> WASM -> host -> renderer.

## O Que Ja Foi Portado Para Rust

O que realmente existe hoje em `crates/unode/src/core`:

### 1. AST canonica

Ja existe uma AST grande e semanticamente tipada em Rust em `crates/unode/src/core/ast.rs`.

Pontos positivos:

- enums no lugar de varias strings soltas;
- `UiExpr<T>` e `OneOrExpr<T>` ja modelam `literal`, `binding` e `param`;
- os node kinds principais ja existem;
- nao ha contaminacao obvia de Mugens, DOM ou TUI dentro do core.

Isso ja esta mais proximo do desenho desejado do que a stack antiga.

### 2. DSL de autoria

`crates/unode/src/core/dsl.rs` ja tem bastante trabalho pronto:

- builders fluentes;
- traits de composicao (`IntoNode`, `IntoChildren`, etc.);
- suporte razoavel para `Option`, arrays, `Vec` e iteradores;
- helpers de expressao;
- macro minima para composicao.

Essa parte ja aponta para uma `unode-sdk` ergonomica, mesmo que hoje ela ainda esteja acoplada ao crate `unode`.

### 3. Normalizacao

`crates/unode/src/core/normalize.rs` ja faz uma parte importante do trabalho:

- colapso de literais;
- defaults;
- validacao de identidade global/sibling;
- calculo de reatividade do no e da subarvore;
- producao de structs canonicas normalizadas.

Ou seja: a base do protocolo ja comecou a existir de verdade no Rust.

### 4. Lowering / IR / envelope

Existem `crates/unode/src/core/ir.rs` e `crates/unode/src/core/transport.rs`.

Isso mostra que ja foi iniciada uma separacao entre:

- AST canonica normalizada;
- formato reduzido para transporte/rendering.

Essa direcao pode ser boa, mas hoje ela ainda conflita com a documentacao e com o contrato antigo. Isso precisa ser decidido conscientemente, nao por acidente.

## O Que Ainda Nao Foi Portado

### 1. Ainda falta metade do `unode` core

O roadmap e os docs assumem que `unode` em Rust teria:

- `MemoryStateStore`;
- `ExprResolver`;
- `trackReactiveBindings`;
- contratos de runtime;
- tipos de permissao e `PermissionGuard`;
- i18n core;
- registries;
- testes de paridade com TypeScript.

Nada disso existe hoje em `crates/unode`.

Em outras palavras: o core portado hoje cobre AST/DSL/normalize, mas ainda nao cobre runtime reativo nem boundary de permissao.

### 2. `unode-sdk` ainda nao existe de fato

`crates/unode-sdk/src/lib.rs` ainda esta em placeholder.

Ainda faltam:

- `PluginManifest`;
- `PluginContext`;
- wrappers de host functions;
- macros/boilerplate de export WASM;
- `unode_alloc` / `unode_dealloc`;
- superficie publica para autoria de plugin.

Hoje a DSL existe, mas o SDK de plugin ainda nao.

### 3. `mugens-domain` ainda nao existe de fato

`crates/mugens-domain/src/lib.rs` ainda esta em placeholder.

Ainda faltam:

- DTOs;
- modelos de dominio;
- tipos compartilhados entre bridge, app e plugins;
- definicao clara do que e dado de dominio e do que e apenas view model.

### 4. `mugens-sdk` ainda nao existe de fato

`crates/mugens-sdk/src/lib.rs` ainda esta em placeholder.

Ainda faltam:

- traits de API de dominio;
- metadata de permissao por metodo;
- registro de host functions para web e TUI;
- locale provider;
- sugar de dominio;
- boundary entre dominio Mugens e core generico.

### 5. `unode-web-runtime` ainda nao existe de fato

`crates/unode-web-runtime/src/lib.rs` ainda esta em placeholder.

Ainda faltam:

- bindings JS-friendly;
- ciclo `load/render/dispatch` para browser;
- adaptacao de imports JS -> host functions;
- integracao com a futura execucao WASM no renderer web.

### 6. `renderer` ainda nao existe de fato

`crates/renderer/src/lib.rs` ainda esta em placeholder.

Ainda faltam:

- loader/host de plugin via Wasmtime;
- renderer Ratatui;
- layout engine;
- focus/input/navigation;
- suporte a media;
- loop reativo por patch;
- enforcement de permissao no host.

### 7. `mgn` ainda e so um stub

`crates/mgn/src/main.rs` ainda e um hello world.

## O Que Continua No TypeScript E Ainda Precisa Ser Trazido Ou Redesenhado

Hoje a implementacao TypeScript ainda concentra quase todo o comportamento vivo:

- `ts-implementation/unode/core/state.ts`
- `ts-implementation/unode/core/resolver.ts`
- `ts-implementation/unode/core/runtime.ts`
- `ts-implementation/unode/runtime/*`
- `ts-implementation/unode/registries/*`
- `ts-implementation/unode/core/i18n.ts`
- `ts-implementation/unode/runtime/guard.ts`
- `ts-implementation/app-plugin-shell/*`
- `ts-implementation/app-plugin-renderer/*`
- `ts-implementation/plugins-bridge/*`

Ou seja: o Rust ainda nao substitui o runtime antigo. Ele so comecou a substituir o protocolo.

## Problemas Da Implementacao TypeScript Que Precisam Ser Corrigidos Na Migracao

Nao faz sentido portar esses problemas para Rust. Eles precisam ser resolvidos no desenho novo.

### 1. Reatividade global demais no renderer web

Em `ts-implementation/app-plugin-shell/ScreenHost.svelte` o renderer usa `rendererStateRevision` global.

Efeito:

- qualquer `state.set()` invalida a tela inteira;
- a granularidade do `StateStore` se perde;
- o `ExprResolver` existe mas nao vira a base do pipeline real.

Esse e um dos principais motivos para a nova arquitetura precisar nascer ja conectando `StateStore + ExprResolver + trackReactiveBindings`.

### 2. Resolucao de tela em `onMount`

`ScreenHost.svelte` ainda resolve tela em `onMount()` e em `afterNavigate()`.

Problemas:

- `preloadData` nao ajuda em telas de plugin;
- o fluxo foge do pipeline normal do SvelteKit;
- loading e navegacao ficam piores do que precisariam.

O novo desenho deve tratar resolucao de tela como etapa de runtime/host, nao como efeito tardio do componente.

### 3. `ensurePluginsActivated()` ainda refaz trabalho demais

Em `ts-implementation/plugins-bridge/runtimeInstance.ts`, a ativacao builtin e cacheada, mas a parte de runtime plugins ainda chama leitura de registry toda vez:

- `fetch('/plugins/registry.json', { cache: 'no-store' })`
- leitura de `localStorage`
- revarredura de URLs dinamicas

Mesmo que plugins ja ativados nao sejam reimportados, ainda existe trabalho desnecessario por chamada.

### 4. Loader sem sandbox forte

`ts-implementation/unode/runtime/loader.ts` ainda usa `import()` de ESM.

Isso significa que o plugin antigo vive no mesmo ambiente JavaScript do host e depende de convencao/guard, nao de isolamento forte.

O salto para WASM precisa eliminar essa ambiguidade:

- plugin sem acesso a memoria do host;
- host functions como unico boundary;
- permissao negada significando capability ausente, nao apenas wrapper que lanca erro.

### 5. Mistura de concerns entre core, host e app

Hoje existe bastante logica espalhada entre:

- `unode/runtime/context.ts`
- `unode/runtime/guard.ts`
- `plugins-bridge/*`
- `app-plugin-shell/*`
- `app-plugin-renderer/*`

O resultado e que:

- permissao core e permissao de dominio aparecem em lugares diferentes;
- renderer web conhece detalhes demais do runtime;
- app shell conhece detalhes demais da tela de plugin;
- o boundary entre `unode`, `bridge` e `renderer` ainda esta poroso.

### 6. `load()` e `render()` ainda nao sao o contrato dominante real

O design-alvo tem separacao forte entre:

- `load()` para buscar dados;
- `render()` para gerar AST imutavel.

Mas o caminho ativo do runtime ainda nao e esse de forma consistente.

Por exemplo, `ts-implementation/unode/runtime/runtime.ts` resolve tela via `route.render(...)` e depois `composeScreen(...)`, enquanto a API ideal descrita em `ts-implementation/unode/core/runtime.ts` ja fala em `PluginRoute.load()` + `PluginRoute.render()`.

Ou seja: a arquitetura desejada ja existe nos tipos, mas nao domina o caminho real de execucao.

### 7. Renderer ainda carrega detalhes web-especificos demais

No renderer TypeScript atual ainda existem dependencias diretas de:

- `window.innerWidth`
- `document`
- `HTMLElement`
- `IntersectionObserver`
- efeitos de DOM no proprio codigo de no

Isso e normal para o renderer web, mas nao pode contaminar o core nem virar pressuposto do protocolo.

### 8. Modelo de permissao ainda esta mais grosso do que o desejado

Hoje o guard em TS protege API groups e alguns builtins, mas o modelo final precisa ser mais forte:

- metadata por metodo;
- default deny real;
- filtro de imports/host functions na instanciacao;
- enforcement identico entre web e TUI;
- origem HTTP aprovada por permissao, nao por convencao solta.

## Divergencias E Pendencias No Codigo Rust Atual

Mesmo dentro do que ja foi portado, existem algumas divergencias importantes.

### 1. O formato canonico ainda nao esta travado

Os docs e a implementacao TypeScript assumem AST canonica com campos como:

- `kind`
- `_key`
- `_reactivity`
- `_subtreeReactivity`
- `_staticFields`

Mas o Rust hoje tem dois formatos diferentes:

- `CanonicalScreen`/`CanonicalUiNode` com metadata serializada como `key`, `reactivity`, `subtreeReactivity`, `staticFields`;
- `IrScreen`/`IrNode` com formato compacto `t/p/c` e metadata `_k`, `_r`, `_sr`, `sf`.

Isso significa que hoje ainda nao esta decidido qual e:

- o contrato publico do protocolo;
- o formato de transporte entre plugin e renderer;
- o formato interno otimizado de rendering.

Essa decisao precisa ser tomada antes de espalhar API nova pelo workspace.

### 2. `_staticFields` ainda nao foi implementado de verdade

O Rust ja calcula reatividade, mas `collect_static_fields_map()` atualmente retorna mapa vazio.

Entao hoje:

- os docs dizem que o normalizer computa campos estaticos;
- a implementacao TS faz isso;
- a implementacao Rust ainda nao faz.

Isso importa porque `_staticFields` e uma das pecas que permitem renderer evitar resolucao desnecessaria em subarvores estaticas.

### 3. Falta de testes de paridade

O roadmap fala explicitamente em validar Rust contra o normalize do TypeScript.

Hoje:

- `cargo test` passa;
- mas passa com zero testes;
- nao existe validacao de compatibilidade do protocolo;
- nao existe golden test;
- nao existe diff entre normalize TS e normalize Rust.

Sem isso, qualquer mudanca no core Rust corre risco de divergir silenciosamente do comportamento atual.

### 4. Politica de identidade ainda esta inconsistente

Hoje existem pelo menos tres narrativas diferentes no repositorio:

- `docs/AST.md` ainda fala em fallback estrutural para nos sem `id`;
- `ts-implementation/docs/unode/CURRENT-STATE.md` diz que identity agora deve ser explicita;
- o Rust atual usa fallback estrutural para alguns nos, mas exige `id` estavel para varios nos interativos/stateful.

Antes de continuar a migracao, precisamos travar uma politica unica para:

- quando fallback estrutural e permitido;
- quais nos exigem id explicito;
- o que conta como identity de reconciliacao vs identity semantica.

### 5. O lowering para IR pode ser uma boa ideia, mas nao deve substituir o core sem decisao

Ter uma IR compacta para renderer/transporte pode ser bom por custo e ergonomia.

Mas isso precisa ficar explicitamente separado em tres camadas:

- AST canonica publica;
- IR interna opcional;
- envelope de transporte.

Se isso nao for explicitado agora, a migracao corre risco de trocar o protocolo sem perceber.

## Separacao Recomendada De Responsabilidades

### `unode`

Deve conter apenas o que for completamente generico:

- AST canonica;
- normalize;
- `MemoryStateStore`;
- `ExprResolver`;
- `trackReactiveBindings`;
- tipos genericos de runtime;
- tipos de permissao e `PermissionGuard`;
- i18n core;
- talvez IR, mas somente se ficar claro que e interna ao ecossistema `unode`.

Nao deve conter:

- DTOs Mugens;
- sugar de dominio;
- host functions de dominio;
- detalhes de renderer web/TUI;
- chrome do app.

### `unode-sdk`

Deve ser a superficie de autoria de plugin:

- DSL publica;
- macros/exports;
- `PluginContext`;
- wrappers de host functions;
- manifest builder;
- tipos ergonomicos para action/load/render.

Nao deve carregar logica de renderer nem detalhes do app Mugens.

### `mugens-domain`

Deve conter somente:

- DTOs;
- tipos de entidade/response;
- modelos de dominio compartilhados.

Nao deve conter:

- sugar de UI;
- runtime de plugin;
- renderer;
- host function registration.

### `mugens-sdk`

Deve conter o bridge app-specific:

- traits de API de dominio;
- metadata de permissao por metodo;
- bindings/registration de host functions;
- locale provider;
- sugar de dominio baseada em `unode`.

### `renderer`

Deve conter:

- execucao de plugin;
- state ownership do host;
- patch loop;
- layout/render/input/focus;
- enforcement final de permissao;
- adaptacao de host functions para a plataforma concreta.

Nao deve conter DTOs Mugens nem definicao de protocolo.

## Ordem Recomendada Das Proximas Etapas

Antes de sair implementando mais codigo, a ordem mais segura parece ser:

1. travar o contrato do protocolo:
   - AST canonica publica versus IR interna;
   - nomes de metadata;
   - politica de identidade;
2. fechar o `unode` core faltante:
   - `MemoryStateStore`
   - `ExprResolver`
   - `trackReactiveBindings`
   - `PermissionGuard`
   - testes de paridade com TS
3. extrair DSL publica para `unode-sdk`;
4. criar `mugens-domain` com DTOs limpos;
5. criar `mugens-sdk` com traits, permissao e host functions;
6. criar `unode-web-runtime` e `renderer`;
7. so depois migrar plugins e telas reais.

## Conclusao

O estado atual e melhor do que um repositorio vazio, mas ainda bem distante de uma migracao funcional.

O que ja existe em Rust e importante:

- AST;
- DSL;
- normalize;
- lowering/transport.

Mas o que ainda falta e justamente o que transforma isso em sistema executavel:

- runtime;
- reatividade;
- permissao;
- sandbox;
- SDK;
- bridge de dominio;
- renderers.

O principal cuidado daqui para frente deve ser evitar dois erros:

- portar cegamente os acoplamentos do TypeScript;
- deixar o protocolo Rust nascer divergente do contrato que os docs dizem que a `unode` quer ter.

Se a gente travar primeiro os boundaries e o contrato canonico, a migracao fica muito mais segura. Se pular essa etapa, a chance de retrabalho entre `unode`, `unode-sdk`, `mugens-sdk` e `renderer` fica alta.
