/**
 * debug-reactive.ts
 *
 * Exemplo de tela com reatividade.
 * Mostra como bindings aparecem no AST e como _reactivity é propagado.
 *
 * Rodar com:
 *   deno run --allow-write debug-reactive.ts
 *   bun run debug-reactive.ts
 *   npx tsx debug-reactive.ts
 */

import { normalizeScreen } from './normalize';
import { screenNodeToJson, debugScreen } from './transport';
import { ui, expr } from './dsl';
import { MemoryStateStore } from './state';

// ─────────────────────────────────────────────────────────────────────────────
// Tela com três padrões de reatividade:
//
//  1. Texto estático — string literal, _reactivity: 'static'
//  2. Texto reativo  — binding para state path, _reactivity: 'reactive'
//  3. Condicional    — condição é um binding, _reactivity: 'conditional'
//  4. Input          — value é binding (lê do state), _reactivity: 'reactive'
//  5. Disclosure     — binding controla expanded, _reactivity: 'reactive'
// ─────────────────────────────────────────────────────────────────────────────

const screen = ui.screen(
  {
    id: 'reactive-demo',
    title: 'Reactive Demo',
    initialState: {
      // Esses valores são mesclados no StateStore antes do primeiro render.
      // Os bindings abaixo lêem desses paths.
      'work.title':     'Berserk',
      'work.year':      1989,
      'ui.favorited':   false,
      'ui.showDetails': false,
      'ui.chaptersLoaded': 42,
    },
  },
  [
    // ── 1. Texto estático — nenhum binding, _reactivity: 'static'
    ui.text('Detalhes da obra', {
      id: 'section-heading',
      role: 'heading',
    }),

    // ── 2. Textos reativos — leem do StateStore
    ui.stack({ id: 'meta-stack', gap: 'sm' }, [
      ui.text(expr.binding('work.title'), {
        id: 'work-title',
        role: 'title',
        emphasis: 'strong',
        // _reactivity: 'reactive' — muda quando 'work.title' muda
      }),

      ui.value(expr.binding('work.year'), 'number', {
        id: 'work-year',
        role: 'caption',
        tone: 'muted',
        // _reactivity: 'reactive' — muda quando 'work.year' muda
      }),

      // Texto estático — mesmo ao lado de reativos
      ui.text('Capítulos carregados:', {
        id: 'chapters-label',
        role: 'label',
        // _reactivity: 'static'
      }),

      ui.value(expr.binding('ui.chaptersLoaded'), 'number', {
        id: 'chapters-count',
        role: 'body',
        // _reactivity: 'reactive'
      }),
    ]),

    // ── 3. Conditional — condição é um binding
    // _reactivity: 'conditional' no ConditionalNode
    // O renderer re-avalia quando 'ui.favorited' muda
    ui.when(
      expr.binding<boolean>('ui.favorited'),
      ui.stack({ id: 'favorited-state', gap: 'xs' }, [
        ui.badge('Favoritado', 'success', { id: 'favorited-badge' }),
        ui.action('Remover dos favoritos', {
          type: 'catalog.removeFavorite',
          confirm: { message: 'Remover dos favoritos?' },
        }, {
          id: 'remove-favorite-action',
          intent: 'danger',
          variant: 'button',
        }),
      ]),
      // else branch — mostrado quando não favoritado
      ui.action('Favoritar', {
        type: 'catalog.addFavorite',
      }, {
        id: 'add-favorite-action',
        intent: 'secondary',
        variant: 'button',
      }),
      { id: 'favorite-conditional' }
    ),

    // ── 4. Input reativo — value lê do state
    ui.input({
      id: 'search-input',
      name: 'searchQuery',
      inputKind: 'text',
      label: 'Buscar capítulos',
      value: expr.binding('searchQuery'),
      placeholder: 'Digite para buscar...',
      // _reactivity: 'reactive' — value binding
    }),

    // ── 5. Disclosure — binding controla collapsed/expanded
    // O binding 'ui.showDetails' é lido e escrito pelo renderer
    ui.disclosure(
      {
        id: 'details-disclosure',
        binding: 'ui.showDetails',
        label: 'Mostrar mais detalhes',
        labelExpanded: 'Ocultar detalhes',
        // _reactivity: 'reactive' — label muda, binding é rastreado
      },
      [
        // Conteúdo colapsável — só renderizado quando expanded
        ui.stack({ id: 'details-content', gap: 'sm' }, [
          ui.text('Serialização completa, mangá seinen.', {
            id: 'details-text',
            role: 'body',
            tone: 'muted',
            // _reactivity: 'static' — texto literal
          }),
        ]),
      ]
    ),
  ]
);

// ─────────────────────────────────────────────────────────────────────────────
// Normalizar e inspecionar
// ─────────────────────────────────────────────────────────────────────────────

const normalized = normalizeScreen(screen);

console.log('\n=== AST normalizada ===\n');
debugScreen(normalized, 'reactive-demo');

// ─────────────────────────────────────────────────────────────────────────────
// Mostrar _reactivity de cada nó do primeiro nível
// ─────────────────────────────────────────────────────────────────────────────

console.log('\n=== _reactivity por nó ===\n');
for (const child of normalized.children) {
  console.log(
    `  [${child.kind.padEnd(12)}] id: ${(child._key).padEnd(25)} ` +
    `_reactivity: ${child._reactivity.padEnd(12)} ` +
    `_subtreeReactivity: ${child._subtreeReactivity}`
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Simular state store e mostrar valores resolvidos
// ─────────────────────────────────────────────────────────────────────────────

const state = new MemoryStateStore(normalized.initialState as Record<string, unknown>);

console.log('\n=== State inicial ===\n');
console.log(JSON.stringify(state.snapshot(), null, 2));

// Simula uma mudança de estado — como se o usuário tivesse clicado em "Favoritar"
state.set('ui.favorited', true);
state.set('work.title', 'Berserk — Edição Deluxe');

console.log('\n=== State após interação ===\n');
console.log(JSON.stringify(state.snapshot(), null, 2));

// ─────────────────────────────────────────────────────────────────────────────
// Serializar para JSON (o que seria mandado pela rede / salvo em arquivo)
// ─────────────────────────────────────────────────────────────────────────────

const json = screenNodeToJson(screen, {
  screenKind: 'reactive-demo',
  pretty: true,
});

console.log('\n=== JSON do envelope (primeiros 500 chars) ===\n');
console.log(json.slice(0, 500) + '\n...');

// Salvar o JSON completo
try {
  // Deno
  if (typeof Deno !== 'undefined') {
    await Deno.writeTextFile('./reactive-screen.json', json);
    console.log('\nSalvo em: ./reactive-screen.json');
  }
  // Node / Bun
  else {
    const { writeFileSync } = await import('node:fs');
    writeFileSync('./reactive-screen.json', json);
    console.log('\nSalvo em: ./reactive-screen.json');
  }
} catch (e) {
  console.log('\nNão foi possível salvar o arquivo:', e);
}

// ─────────────────────────────────────────────────────────────────────────────
// O que esperar no JSON gerado:
//
//  Nós estáticos:
//    "_reactivity": "static",
//    "_subtreeReactivity": "static",
//    "_staticFields": { "content": "Detalhes da obra", "role": "heading" }
//
//  Nós reativos (binding):
//    "_reactivity": "reactive",
//    "_subtreeReactivity": "reactive",
//    "content": { "kind": "binding", "path": "work.title" }
//    "_staticFields": { "role": "title", "emphasis": "strong" }
//
//  Conditional:
//    "_reactivity": "conditional",
//    "_subtreeReactivity": "conditional",
//    "condition": { "kind": "binding", "path": "ui.favorited" }
//
//  Stack com filho reativo:
//    "_reactivity": "static",       ← o próprio stack não tem bindings
//    "_subtreeReactivity": "reactive" ← mas um filho tem
// ─────────────────────────────────────────────────────────────────────────────