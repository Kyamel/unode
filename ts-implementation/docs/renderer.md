Sim, e é mais do que parece à primeira vista. Deixa eu pensar por camadas.

---

## O que definitivamente não dá para compartilhar

Tudo que toca output é específico de plataforma por definição. Svelte emite DOM, Notcurses emite células de terminal. Não tem abstração que valha a pena ali — qualquer tentativa vira uma camada vazia.

---

## O que já é compartilhado (e você provavelmente não percebeu)

O `normalize.ts` inteiro. Ele já é puro TypeScript sem dependência de DOM ou terminal. Os dois renderers deveriam chamar `normalizeScreen()` antes de montar qualquer coisa. No TUI isso é explícito no plano. No Svelte hoje isso acontece, mas o resultado (`_reactivity`, `_staticFields`, `dependenciesOf`) é ignorado — que é exatamente o bug central da Part 1 do plano Svelte.

O `resolver.ts` (`DefaultExprResolver` com `track`, `clearTracking`, `subscribersOf`). O TUI usa diretamente. O Svelte deveria usar para substituir o `resolve.ts` atual — que reimplementa a lógica de resolução de expressões sem o sistema de tracking de dependências.

O `config.ts` com `RendererConfig`, `webRendererConfig`, `tuiRendererConfig`, e `createRendererConfig`. Você já tem isso compartilhado. O `tuiRendererConfig` usa breakpoints em colunas de terminal em vez de pixels — essa é a única diferença semântica.

---

## O que vale extrair agora como camada compartilhada

Há três coisas concretas que não existem ainda mas deveriam ser código compartilhado:

**1. Resolução de layout semântico**

Os dois renderers precisam traduzir `gap: 'md'` para um valor concreto, `columns` responsivo para um número de colunas dado um viewport, e `AspectRatio` para uma proporção. Hoje o Svelte faz isso inline em CSS e o TUI faria em TypeScript no layout engine. A lógica de decisão é a mesma:

```typescript
// src/lib/unode/core/renderer/layout-semantics.ts
// Compartilhado entre web e TUI

export function resolveGap(gap: Gap | undefined, config: RendererConfig): number {
  // Retorna um número semântico — o renderer decide a unidade (px, rem, células)
  const scale = { none: 0, xs: 1, sm: 2, md: 3, lg: 5 };
  return scale[gap ?? 'md'] ?? scale.md;
}

export function resolveGridColumns(
  columns: ResponsiveGridColumns | undefined,
  viewportWidth: number,         // px no web, colunas de terminal no TUI
  breakpoints: RendererBreakpoints
): number {
  if (!columns) return 1;
  if (viewportWidth >= breakpoints.xl && columns.xl) return columns.xl;
  if (viewportWidth >= breakpoints.lg && columns.lg) return columns.lg;
  if (viewportWidth >= breakpoints.md && columns.md) return columns.md;
  if (viewportWidth >= breakpoints.sm && columns.sm) return columns.sm;
  return columns.base ?? 1;
}

export function resolveAspectRatio(ratio: AspectRatio | undefined): number {
  // width / height
  const ratios = { square: 1, poster: 2/3, video: 16/9, auto: 0 };
  return ratios[ratio ?? 'auto'] ?? 0;
}
```

O `CoreGridNode.svelte` já tem `resolveCols` — ela pode ser deletada e substituída por isso. O layout engine do TUI usa a mesma função com `ncDimensions().cols` como viewport width.

**2. Formatação de valores**

O `formatValue` dentro do `CoreUiRenderer.svelte` usa `Intl.NumberFormat` e `Intl.DateTimeFormat`. Isso é TypeScript puro e funciona no Deno também. Deveria sair do componente Svelte e virar uma função utilitária compartilhada:

```typescript
// src/lib/unode/core/renderer/format-value.ts

export function formatNodeValue(
  node: CanonicalNode<ValueNode>,
  resolvedValue: Primitive,
  locale: string
): string {
  if (resolvedValue === null || resolvedValue === undefined || resolvedValue === '') return '';

  switch (node.format) {
    case 'currency':
      return new Intl.NumberFormat(locale, {
        style: 'currency',
        currency: node.currencyCode ?? 'USD'
      }).format(Number(resolvedValue));

    case 'number':
    case 'percent':
      return new Intl.NumberFormat(locale, {
        style: node.format === 'percent' ? 'percent' : 'decimal'
      }).format(Number(resolvedValue));

    case 'date':
    case 'datetime':
      const date = new Date(String(resolvedValue));
      if (!isNaN(date.getTime())) {
        return new Intl.DateTimeFormat(locale, {
          dateStyle: 'medium',
          timeStyle: node.format === 'datetime' ? 'short' : undefined
        }).format(date);
      }
      return String(resolvedValue);

    case 'bytes':
      return formatBytes(Number(resolvedValue));

    case 'duration':
      return formatDuration(Number(resolvedValue));

    default:
      return String(resolvedValue);
  }
}
```

**3. Validação semântica de ação em dev**

Ambos os renderers vão querer avisar quando um plugin tenta usar `unode.setState` sem declarar o path no `initialState`, ou quando um `DisclosureNode` usa um binding que não foi inicializado. Essa validação é pura lógica sobre o AST e o estado — não tem nada de plataforma. Uma função `validateScreenInDev(screen, config)` que roda depois de `normalizeScreen` e emite warnings no console serviria os dois renderers.

---

## O que não vale extrair agora

A lógica de renderização de cada nó individual não tem abstração útil entre web e TUI. A tentação é criar algo como `NodeRenderer` abstrato que cada plataforma implementa, mas isso vira uma interface de 25 métodos que só existe para ser implementada duas vezes de formas completamente diferentes. Não agrega nada além de burocracia.

O mesmo vale para navegação (`Navigator`), foco, e eventos de input — os conceitos são paralelos mas as implementações são ortogonais.

---

## Estrutura de arquivos resultante

```
src/lib/unode/core/
  ast.ts                    ← já compartilhado
  dsl.ts                    ← já compartilhado
  normalize.ts              ← já compartilhado
  immutable.ts              ← já compartilhado
  runtime.ts                ← já compartilhado
  i18n.ts                   ← já compartilhado
  resolver.ts               ← já compartilhado

  renderer/                 ← novo, compartilhado entre web e TUI
    config.ts               ← já existe, manter aqui
    layout-semantics.ts     ← novo: resolveGap, resolveGridColumns, resolveAspectRatio
    format-value.ts         ← novo: formatNodeValue com Intl
    dev-validation.ts       ← novo: validateScreenInDev

src/lib/unode-web-renderer/ ← específico do Svelte
  ...

apps/tui/src/renderer/     ← específico do TUI
  ...
```

O critério simples para qualquer nova função: **se ela não importa nada de `svelte`, nada de DOM, e nada de Notcurses/terminal, ela provavelmente é compartilhada**. Se importa qualquer uma dessas coisas, é específica de plataforma.