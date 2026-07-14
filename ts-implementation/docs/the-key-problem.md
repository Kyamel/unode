Esse medo é legítimo, mas o problema que você está descrevendo é específico — e tem uma solução mais cirúrgica do que obrigar key em todo nó.

---

## Onde key instável realmente causa bug

Key instável causa problema em exatamente dois contextos:

**1. Listas dinâmicas** — quando o renderer reconcilia `ListNode.items` ou `GridNode.children` mapeados de dados, uma key baseada em índice posicional faz o renderer reusar o DOM/célula errada quando a ordem muda ou um item é removido do meio.

**2. `ConditionalNode`** — quando a condição muda de branch, o renderer precisa de key para saber se deve reusar ou remontar os nós dos dois branches.

Em todos os outros casos — `StackNode`, `InlineNode`, `SectionNode`, estrutura estática — key instável não causa bug porque a estrutura não muda entre renders do mesmo load cycle. O renderer não precisa reconciliar nós que nunca mudam de posição.

Plugins carregados em runtime com ordem arbitrária também não são problema de key — são problema de **slot** e de **ordering na SlotRegistry**. A key dentro da subárvore de um plugin não precisa ser globalmente única entre plugins, só localmente estável dentro daquela subárvore.

---

## A regra mínima

Em vez de obrigar key em todo nó, você pode obrigar key só onde instabilidade causa bug real. Isso é exprimível no tipo:

```typescript
// ItemNode sempre precisa de key — está em listas dinâmicas por definição
export interface ItemNode extends NodeBase {
  readonly kind: 'item';
  readonly key: string;  // required, não opcional
  readonly primary: readonly UiNode[];
  // ...
}

// ListNode de items dinâmicos — key nos items, não na lista
export interface ListNode extends NodeBase {
  readonly kind: 'list';
  readonly items: readonly ItemNode[];  // ItemNode.key é required
  // ...
}

// ConditionalNode — key nos branches quando o conteúdo é dinâmico
// Mas o próprio ConditionalNode não precisa de key
export interface ConditionalNode extends NodeBase {
  readonly kind: 'conditional';
  readonly condition: BoolOrExpr;
  readonly then: UiNode;
  readonly else?: UiNode;
}
```

```typescript
// GridNode com children mapeados de dados — precisa de key nos filhos
// mas você não pode enforçar isso no tipo sem perder generalidade
// A solução é documentar + lint rule, não tipo obrigatório
export interface GridNode extends NodeBase {
  readonly kind: 'grid';
  readonly children: readonly UiNode[];
}
```

Para o `GridNode` com children dinâmicos, você não consegue enforçar key no tipo sem criar `DynamicGridNode` separado, o que é overhead. A solução aqui é diferente — o normalizer emite um warning em dev quando detecta children sem key em nós que têm `continuation` (que é o sinal de que o conteúdo é dinâmico):

```typescript
function normalizeGrid(node: GridNode, ctx: NormalizeContext): CanonicalNode<GridNode> {
  if (import.meta.env.DEV && node.continuation && node.children.some(c => !c.key)) {
    console.warn(
      `[unode] GridNode at ${ctx.path} has continuation but children without keys. ` +
      `Keys are required for stable reconciliation in dynamic collections.`
    );
  }
  // ...
}
```

---

## O que o normalizer pode gerar com segurança

Para tudo que não é lista dinâmica nem conditional, o normalizer pode gerar keys por path sem risco:

```typescript
// path: "screen.c0.c1.c2"
// Estável porque:
// 1. A estrutura do screen não muda entre state changes no mesmo load cycle
// 2. Cada load cycle produz um novo screen do zero via render()
// 3. A ordem de filhos numa estrutura estática não muda
```

A key gerada por path é estável dentro de um load cycle. Entre load cycles (nova navegação), o renderer já desmonta e remonta tudo — então key não precisa ser estável entre load cycles.

O único caso onde path-key falha é exatamente o que você identificou: listas com items que mudam de posição ou são removidos do meio. Mas esses são exatamente os `ItemNode` onde você já tem `key` required no tipo.

---

## Recomendação concreta

Três mudanças no que você tem:

**1.** `ItemNode.key` vira `required` no tipo. É o único nó onde key instável causa bug garantido.

**2.** O normalizer volta a gerar keys por path para todos os outros nós. Remove a obrigatoriedade de key nos builders exceto `item()`.

**3.** Warning em dev quando `GridNode` ou `ListNode` com `continuation` tem filhos sem key explícita. Isso captura o outro caso problemático sem tornar o tipo mais complexo.

O resultado é que o caminho feliz — estrutura estática, sem listas dinâmicas — não precisa de nenhuma key. O plugin de listas de mangas coloca `key: work.id` nos items porque o tipo obriga. O normalizer cuida do resto.