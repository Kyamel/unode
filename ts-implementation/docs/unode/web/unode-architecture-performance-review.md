# unode / plugins-bridge / web renderer

## Escopo

Este documento faz uma analise arquitetural do estado atual de:

- `src/lib/unode`
- `src/lib/plugins-bridge`
- renderer web em `src/lib/widgets/app-plugin-renderer` e `src/lib/widgets/app-plugin-shell`

Tambem analisa o comportamento de desempenho percebido ao abrir telas de plugin, em particular o loading visivel antes da tela aparecer.

## Metodo

Esta analise foi feita por inspecao de codigo e pelos contratos atuais do runtime/renderer. Nao houve profiling no browser com flamechart nesta rodada, entao as conclusoes de desempenho abaixo sao inferencias fortes baseadas no fluxo real do codigo, nao microbenchmarks.

## Resumo executivo

Conclusao curta:

- A maior parte de `plugins-bridge` esta no lugar certo e deve continuar fora do `unode`.
- Existe um subconjunto pequeno e claramente generico no bridge que deveria sair dali.
- O atraso perceptivel ao abrir uma tela de plugin nao parece ser um problema "natural" do Svelte ou do DOM; ele decorre principalmente do desenho atual do fluxo de carregamento.
- O principal gargalo arquitetural e que a tela de plugin nao usa o pipeline nativo de `load`/`preloadData` do SvelteKit. Ela resolve a tela so depois que o componente monta, via `ScreenHost`.
- Existe tambem pelo menos um problema de implementacao importante: o runtime dynamic registry e buscado de novo em toda chamada de `ensurePluginsActivated()`, inclusive em navegacao, sidebar e command palette.
- No renderer, a granularidade da reatividade do `StateStore` e perdida: qualquer alteracao de estado invalida o renderer inteiro da tela, nao apenas os bindings afetados.

## 1. O que em `plugins-bridge` deveria sair dali

### 1.1 Deve continuar no `plugins-bridge`

Esses itens sao claramente especificos do app Mugen e estao no lugar certo:

- `src/lib/plugins-bridge/capabilities.ts`
- `src/lib/plugins-bridge/models.ts`
- `src/lib/plugins-bridge/host.ts`
- `src/lib/plugins-bridge/hostApi.ts`
- `src/lib/plugins-bridge/guard.ts`
- `src/lib/plugins-bridge/components/*`
- `src/lib/plugins-bridge/domains/manga/*`

Motivo:

- definem APIs de dominio (`catalog`, `users`, `reader`, `auth`)
- definem modelos de dominio (`WorkSummary`, `ChapterSummary`, etc.)
- definem sugar/view-models de dominio (`workList`, `chapterList`, `workMetadata`, `workBanner`)

Isso bate com a propria direcao documentada do `unode`: dominio da app vive no bridge, nao no core.

### 1.2 Deveria subir para `unode`

#### `src/lib/plugins-bridge/navigation.ts`

Hoje:

- o arquivo so encapsula a acao built-in `unode.navigate`
- nao adiciona nenhum comportamento de dominio

Evidencia:

- `src/lib/plugins-bridge/navigation.ts:3`

Recomendacao:

- mover para `unode`, por exemplo como helper de authoring:
  - `createNavigateAction(...)`
  - ou `actions.navigate(...)`

Justificativa:

- ele nao e "bridge Mugen"
- ele e uma convenience API em cima de um built-in do proprio core

### 1.3 Nao deveria estar no `plugins-bridge`, mas tambem nao no `unode core`

#### `src/lib/plugins-bridge/screen-chrome/route-tabs.ts`

Hoje:

- define um meta contract generico de `routeTabs`
- faz parse/serialize de `meta.routeTabs`
- e usado por `ScreenHost` para renderizar `RouteTabsLayout`

Evidencia:

- `src/lib/plugins-bridge/screen-chrome/route-tabs.ts:22`
- `src/lib/widgets/app-plugin-shell/ScreenHost.svelte:7`
- `src/lib/widgets/app-plugin-shell/ScreenHost.svelte:222`

Minha leitura:

- isso nao e dominio Mugen
- mas tambem nao e um conceito do `unode core`
- isso e "shell UI / web renderer chrome"

Recomendacao:

- mover para a camada do renderer web/shell
- por exemplo:
  - `src/lib/unode-web/screen-chrome/route-tabs.ts`
  - ou `src/lib/widgets/app-plugin-shell/route-tabs.ts`

Se entrar no `unode core`, o core comeca a aprender conceitos de shell visual que deveriam continuar opcionais.

### 1.4 Duplicacao que deveria ser eliminada

#### `src/lib/plugins-bridge/rendererConfig.ts`

Hoje:

- o bridge replica merge/config logic que ja existe em `src/lib/unode/renderer/config.ts`

Evidencia:

- `src/lib/plugins-bridge/rendererConfig.ts:18`
- `src/lib/unode/renderer/config.ts:87`

Recomendacao:

- remover esse wrapper do bridge
- consumir `createRendererConfig(...)` direto do `unode`
- se precisar de presets de app, manter so um objeto de override, nao uma segunda fabrica

## 2. O que nao deveria subir para `unode`

### Component sugar de dominio

Arquivos como:

- `src/lib/plugins-bridge/components/work-list/component.ts`
- `src/lib/plugins-bridge/components/chapter-list/component.ts`
- `src/lib/plugins-bridge/components/work-banner/component.ts`
- `src/lib/plugins-bridge/components/work-metadata/component.ts`

devem permanecer fora do core.

Motivo:

- apesar de usarem `ui.*` do `unode`, continuam sendo "design sugar + semantic sugar" do dominio Mugen
- a API deles embute conceitos como `work`, `chapter`, `staff`, `metadata`, `cover`, `taxonomy`

Se isso subir para o core, o `unode` deixa de ser agnostico ao dominio.

## 3. Analise de desempenho atual

## 3.1 O loading percebido ao abrir uma tela de plugin

O comportamento relatado faz sentido com o codigo atual.

Ao entrar em uma rota de plugin:

1. a rota SvelteKit em si carrega so um componente fino:
   - `src/routes/app/(shell)/[...pluginPath]/+page.svelte:1`
2. esse componente monta `ScreenHost`
3. `ScreenHost` liga `loading = true` imediatamente:
   - `src/lib/widgets/app-plugin-shell/ScreenHost.svelte:149`
   - `src/lib/widgets/app-plugin-shell/ScreenHost.svelte:152`
4. so depois disso ele chama:
   - `ensurePluginsActivated(host)`
   - `runtime.resolveScreen(...)`
5. enquanto isso, ele renderiza o banner:
   - `src/lib/widgets/app-plugin-shell/ScreenHost.svelte:248`

Ou seja: o loading state nao e um detalhe visual acidental. Ele e parte estrutural do fluxo atual.

## 3.2 Por que a experiencia fica mais lenta do que uma pagina SvelteKit normal

### Causa principal: a tela de plugin nao participa do pipeline nativo de `load` do SvelteKit

O app ja esta configurado com:

- SPA client-only no shell:
  - `src/routes/app/+layout.ts:1`
- preload de dados em links:
  - `src/app.html:38`

Pelo contrato do SvelteKit:

- links podem disparar `preloadData`
- se a pagina usa `+page.js/+page.ts load`, a navegacao seguinte pode ficar praticamente instantanea

Mas a rota de plugin nao tem `+page.ts` nem `load` algum. O carregamento da tela acontece fora do roteador, dentro de `ScreenHost`, depois da navegacao ja ter acontecido.

Consequencia:

- o preload nativo do SvelteKit nao consegue preaquecer os dados da tela de plugin
- o usuario ve a navegacao acontecer e so entao o runtime comeca a resolver a tela

Essa e a explicacao arquitetural mais forte para a diferenca "pagina Svelte parece instantanea, plugin mostra loading".

## 3.3 Problema concreto de implementacao: registry fetch repetido em toda ativacao

`ensurePluginsActivated()` so ativa os builtin plugins uma vez, mas sempre volta a chamar `activateRuntimePlugins(hostApi)` depois disso:

- `src/lib/plugins-bridge/runtimeInstance.ts:135`
- `src/lib/plugins-bridge/runtimeInstance.ts:141`
- `src/lib/plugins-bridge/runtimeInstance.ts:147`

`activateRuntimePlugins()` por sua vez sempre chama `loadRuntimePluginUrls()`:

- `src/lib/plugins-bridge/runtimeInstance.ts:117`
- `src/lib/plugins-bridge/runtimeInstance.ts:118`

E `loadRuntimePluginUrls()` faz:

- parse de globals
- parse de `localStorage`
- `fetch('/plugins/registry.json', { cache: 'no-store' })`

Evidencia:

- `src/lib/plugins-bridge/runtimeInstance.ts:79`
- `src/lib/plugins-bridge/runtimeInstance.ts:103`

Impacto:

- toda vez que `ensurePluginsActivated()` roda, existe pelo menos a chance de uma ida a rede para o registry
- isso nao acontece so ao abrir uma tela de plugin
- tambem acontece ao recalcular sidebar e command palette:
  - `src/lib/widgets/app-shell/navItems.ts:43`
  - `src/lib/widgets/command-palette/CommandPalette.svelte:174`

Este ponto tem cara de bug de implementacao, nao de tradeoff inevitavel.

## 3.4 Problema arquitetural de cache: telas de plugin nao reaproveitam o cache ja existente da app

O app ja sobe um `queryClient` global:

- `src/routes/app/+layout.svelte:15`

E o catalog repo ja expoe query builders:

- `src/lib/entities/catalog/api/worksRepo.ts:129`
- `src/lib/entities/catalog/api/worksRepo.ts:136`

Mas o bridge chama os metodos "crus" do repo:

- `src/lib/plugins-bridge/hostApi.ts:231`
- `src/lib/plugins-bridge/hostApi.ts:243`

E os plugins consomem isso direto:

- `src/lib/plugins/mangas/browse-hot/data.ts:17`
- `src/lib/plugins/mangas/work-meta/data.ts:15`

Consequencia:

- a tela de plugin nao reaproveita automaticamente o cache de query do host
- ela tende a fazer fetch "frio" mesmo quando o host ja teria mecanismos para resposta instantanea ou stale-while-revalidate

### Exemplo claro

`browse-hot` sempre espera a pagina remota antes de decidir o que renderizar:

- `src/lib/plugins/mangas/browse-hot/index.ts:118`
- `src/lib/plugins/mangas/browse-hot/index.ts:120`

Mesmo quando existe estado persistido, ele ainda bloqueia no fetch remoto para calcular `filterKey` e decidir qual conjunto usar.

Ja `work-meta` esta melhor desenhado:

- usa storage como cache de leitura rapida
- so bate no catalog se nao houver cache

Evidencia:

- `src/lib/plugins/mangas/work-meta/data.ts:27`
- `src/lib/plugins/mangas/work-meta/data.ts:32`

## 3.5 Problema do renderer: granularidade perdida no binding state

O `MemoryStateStore` do core suporta listeners por path e por prefixo:

- `src/lib/unode/core/state.ts:145`
- `src/lib/unode/core/state.ts:157`

Mas no adapter Svelte do renderer, todo o estado e reduzido a um contador global `rendererStateRevision`:

- `src/lib/widgets/app-plugin-shell/ScreenHost.svelte:28`
- `src/lib/widgets/app-plugin-shell/ScreenHost.svelte:69`
- `src/lib/widgets/app-plugin-shell/ScreenHost.svelte:84`
- `src/lib/widgets/app-plugin-shell/ScreenHost.svelte:88`

Na pratica:

- qualquer mudanca em qualquer path chama `rendererStateRevision += 1`
- qualquer binding resolvido via `state.get(...)` depende desse contador unico
- entao qualquer alteracao invalida o renderer inteiro daquela tela

Isso nao afeta tanto o primeiro load, mas afeta:

- formularios
- disclosures
- filtros
- listas com continuation incremental
- qualquer tela com muitos bindings

Isso e uma evidenca de problema de implementacao do renderer, nao do core.

## 3.6 Trabalho redundante em toda navegacao de tela

Cada resolucao de tela:

- instancia um `MemoryStateStore` novo
- executa `load()`
- executa `render()`
- normaliza a AST
- faz `deepFreeze` em toda a arvore

Evidencia:

- `src/lib/unode/registries/routes.ts:76`
- `src/lib/unode/registries/routes.ts:78`
- `src/lib/unode/registries/routes.ts:84`
- `src/lib/unode/core/immutable.ts:11`

Esse custo por si so talvez nao explique ~500ms, mas ele soma com:

- fetch remoto
- ativacao do runtime
- registry fetch
- ausencia de preload

Ou seja: `normalize + freeze` parece custo secundario, nao raiz unica.

## 3.7 Falta cancelamento real de requisicoes

O host/runtime hoje so faz descarte logico por `requestKey`:

- `src/lib/widgets/app-plugin-shell/ScreenHost.svelte:150`
- `src/lib/widgets/app-plugin-shell/ScreenHost.svelte:169`

Mas o contrato ate preve `AbortSignal` em `RouteRenderContext`:

- `src/lib/unode/api/contracts.ts:45`

So que esse `signal` nao e usado no runtime nem nos plugins.

Consequencia:

- navegacoes rapidas continuam fazendo trabalho inutil em background
- o resultado velho e ignorado, mas o custo de rede/CPU continua sendo pago

## 3.8 Custo de runtime vazando para shell global

Sidebar mobile/desktop e command palette recalculam itens do runtime em navegacoes:

- `src/lib/widgets/app-shell/AppSidebarContent.svelte:16`
- `src/lib/widgets/app-shell/AppSidebarContent.svelte:27`
- `src/lib/widgets/app-shell/AppMobileNavBar.svelte:23`
- `src/lib/widgets/command-palette/CommandPalette.svelte:174`

Esses caminhos passam por `ensurePluginsActivated()`, que hoje pode refazer trabalho do registry. Isso significa que o custo do runtime nao esta confinado so a tela de plugin.

## 4. Caminho recomendado para melhorar desempenho

## Prioridade 0: corrigir os gargalos mais provaveis

### 4.1 Nao refazer `loadRuntimePluginUrls()` em toda navegacao

Fazer o registry dinamico ter cache em memoria de sessao.

Opcoes:

- carregar uma vez por sessao
- recarregar so por comando manual / refresh explicito
- em dev, permitir refresh mais agressivo; em prod, cachear

Esperado:

- remove custo de rede e parse repetido
- reduz o tempo ate `resolveScreen`

### 4.2 Integrar telas de plugin ao `load` do SvelteKit

Esse e o maior ganho potencial para UX percebida.

Direcao:

- criar `+page.ts` para `[...pluginPath]`
- resolver `ensurePluginsActivated + resolveScreen` no `load`
- passar o `ResolvedScreen` para `+page.svelte`

Beneficios:

- `data-sveltekit-preload-data="hover"` finalmente passa a ajudar tambem nas telas de plugin
- navegacao entre telas de plugin pode ficar muito mais proxima da experiencia "instantanea"
- o loading state pode virar fallback raro, nao estado normal de entrada

### 4.3 Evitar loading bloqueante quando ja existe conteudo antigo ou cache local

Para rotas como browse/list:

- mostrar imediatamente a ultima tela/cached data
- fazer refresh em background
- atualizar quando os novos dados chegarem

Na pratica: stale-while-revalidate para plugin screens.

## Prioridade 1: corrigir problemas de implementacao do renderer

### 4.4 Restaurar granularidade de reatividade por path

Hoje o renderer perde a granularidade do `StateStore`.

Recomendacao:

- criar subscriptions por binding/path
- ou expor wrappers tipo `subscribe(path)` no context do renderer
- ou gerar stores derivados por path

Objetivo:

- mudar so o subtree afetado por um binding
- nao a tela inteira

### 4.5 Parar de tratar o web renderer como detalhe espalhado em `widgets`

Hoje o renderer real esta fragmentado entre:

- `widgets/app-plugin-renderer`
- `widgets/app-plugin-shell`
- `plugins-bridge/screen-chrome`

Enquanto `src/lib/unode/renderer` contem basicamente config.

Recomendacao:

- consolidar o renderer web como uma camada clara
- por exemplo `src/lib/unode-web-renderer/*`

Isso nao melhora tempo de CPU por si so, mas melhora muito a capacidade de otimizar sem espalhar concerns.

## Prioridade 2: melhorar a estrategia de dados

### 4.6 Dar ao runtime de plugin acesso a cache reutilizavel do host

Opcoes:

- bridge expor um cache/query facade generico
- integrar `host.catalog.*` com query cache interno
- permitir `load()` marcar dados como `stale`, `ttl`, `cacheKey`

Sem isso, plugins tendem a parecer "mais lentos" que telas nativas mesmo quando batem nos mesmos dados.

### 4.7 Adotar `AbortSignal` de verdade

Passar `signal` ate:

- runtime
- host api
- fetch/http
- loaders dos plugins

Beneficio:

- navegacoes rapidas deixam de desperdiçar fetch e CPU

## 5. Recomendacao arquitetural objetiva sobre o bridge

Minha recomendacao final sobre fronteira `plugins-bridge` -> `unode` e:

Mover para `unode`:

- helper de built-in action `navigateAction`

Remover do bridge por duplicacao:

- `rendererConfig.ts`

Mover para a camada do renderer web/shell, nao para o core:

- `screen-chrome/route-tabs.ts`

Manter no bridge:

- capabilities
- models
- host api
- domain guards
- sugar/componentes de dominio

## 6. Diagnostico especifico do loading visivel

Se eu tivesse que apontar uma causa principal para o loading de ~meio segundo que voce percebe, eu priorizaria assim:

1. a tela de plugin carrega fora do `load` do SvelteKit e portanto nao aproveita `preloadData`
2. `ScreenHost` sempre entra em `loading = true` antes de resolver runtime + dados
3. `ensurePluginsActivated()` ainda refaz trabalho demais, incluindo `fetch('/plugins/registry.json', { cache: 'no-store' })`
4. alguns plugins ainda bloqueiam a primeira pintura esperando fetch remoto
5. plugin screens nao reaproveitam o cache de dados ja existente da app

Em outras palavras:

- o problema parece muito mais de arquitetura de carregamento e cache do que de "Svelte renderizar devagar"

## 7. Sequencia pratica sugerida

Se fosse para atacar isso em ordem de ROI:

1. cachear `loadRuntimePluginUrls()` em memoria e parar de buscar o registry em toda ativacao
2. mover a resolucao de tela de plugin para `+page.ts load`
3. introduzir stale-while-revalidate para listas/telas com cache local
4. reescrever o adapter de estado do renderer para subscriptions por path
5. consolidar o renderer web numa camada propria e tirar wrappers duplicados do bridge

## Conclusao

Nao ha evidencia de que o conceito do `unode` seja inerentemente lento. O que existe hoje e um conjunto de escolhas que empurram o carregamento das telas de plugin para depois da navegacao, fora do pipeline otimizado do SvelteKit, e ainda repetem trabalho do runtime mais do que o necessario.

O principal caminho para a UX ficar "instantanea" como uma pagina SvelteKit normal nao e micro-otimizar JSX/Svelte DOM. E:

- preloading real
- cache real
- evitar ativacao repetida
- reduzir invalidacao global no renderer
