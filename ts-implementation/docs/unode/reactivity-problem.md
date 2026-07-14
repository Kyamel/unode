Confirmado. Agora tenho o quadro completo.

---

## O que existe vs o que foi projetado

O `runtime.ts` em `core/runtime` declara `ExprResolver` com `track`, `clearTracking`, `dependenciesOf`, `subscribersOf` — a interface completa para reatividade granular. Mas procurando em todos os arquivos que você mandou, **não existe implementação de `ExprResolver` em lugar nenhum**. É só a interface.

O que existe no runtime real (`registries/routes.ts`) é:

```typescript
const state = new MemoryStateStore();
const renderCtx = createRenderContext(plugin, host, route, state, i18n);
const data = await def.load(renderCtx);
state.mergeData(data);
const body = normalizeScreen(def.render(data, renderCtx));
if (body.initialState) state.mergeData(body.initialState);
return { body, state, ... };
```

O `body` chega normalizado com `_reactivity` e `_staticFields`. O `state` é um `MemoryStateStore`. Mas não existe nada conectando os dois — nenhum código lê `_reactivity` ou `dependenciesOf` para configurar subscriptions. O normalize entrega a análise, o state store existe, mas ninguém usa uma para se inscrever no outro.

O renderer Svelte recebe o `state` e o `body` e ignora os metadados de reatividade, usando o contador global em vez disso. O TUI ainda não existe. Em ambos os casos, a análise do `normalize.ts` está sendo produzida e descartada.

---

## Avaliação honesta

**O que está completo e correto:**

O `MemoryStateStore` (`core/state`, que você não mandou mas referencia em `routes.ts`) — pelos usos que vejo, parece implementado com `get`, `set`, `mergeData`, `subscribe`, `subscribePrefix`, `snapshot`, `reset`. É a fundação certa.

O `normalize.ts` — completo, bem implementado, e produz exatamente o que o `ExprResolver` precisaria consumir.

A interface `ExprResolver` no `runtime.ts` — bem projetada, com os quatro métodos necessários para tracking bidirecional.

O sistema de permissões no `guard.ts` e `context.ts` — funcionando, com testes cobrindo os casos principais.

O runtime de ativação de plugins e resolução de rotas — completo e testado.

**O que é só interface sem implementação:**

`ExprResolver` — declarado, nunca implementado.

**O que é implementação incorreta que compensa a ausência acima:**

O `rendererStateRevision` no Svelte — existe porque sem `ExprResolver`, o renderer não tem como saber quais nós dependem de quais paths.

---

## O que isso significa na prática

O sistema de reatividade tem **duas metades** que nunca foram conectadas:

```
Metade A — produção de metadados (completa)
  normalize.ts → _reactivity, _staticFields, via dependenciesOf interface

Metade B — consumo de metadados (ausente)
  ExprResolver → track(), clearTracking(), subscribersOf()
  Ninguém implementa, ninguém chama
```

Para conectar as duas, o trabalho necessário é:

**1. Implementar `DefaultExprResolver`** — o arquivo `resolver.ts` que eu escrevi no plano inicial ainda não existe. É ~80 linhas com dois Maps bidirecionais (`nodeKey → paths` e `path → nodeKeys`) e os quatro métodos de tracking.

**2. Conectar no `routes.ts`** — depois de `normalizeScreen`, percorrer a árvore e para cada nó com `_reactivity !== 'static'`, chamar `resolver.dependenciesOf(node._key)` e criar subscriptions no `state`.

**3. No Svelte** — substituir o `rendererStateRevision` por `adapter.getPathStore(path)` usando as subscriptions que o resolver já mapeou.

**4. No TUI** — o `ReactiveLoop` do plano TUI lê `resolver.subscribersOf(path)` para saber quais nós repintar quando um path muda.

A boa notícia é que a fundação está toda certa. Não é uma reescrita — é preencher o gap entre as duas metades que foram bem projetadas mas nunca conectadas.