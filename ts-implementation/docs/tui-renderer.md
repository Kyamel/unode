Boa pergunta. Deixa eu pesquisar os renderers nativos mais relevantes.Tenho o quadro completo. A resposta é mais interessante do que parece à primeira vista.

---

## A vantagem real de TS-first via FFI não é o que você pensa

Quando você usa FFI, o renderer nativo vira um **backend de drawing puro** — você passa células, strings, coordenadas, e ele desenha. A linguagem do backend não importa para a sua experiência de desenvolvimento. O que importa é a qualidade da camada de tradução `CanonicalNode → chamadas FFI`, que você vai escrever em TypeScript de qualquer forma.

Então a pergunta real é: **qual biblioteca nativa tem a melhor API para ser chamada via FFI, e qual resolve melhor o seu caso de uso específico** (galeria de capas, layout flexível, imagens).

---

## Os três candidatos sérios

### OpenTUI (Zig, C ABI)

Pontos fortes para o seu caso: layout Yoga já integrado, API orientada a objetos que mapeia bem para a sua AST (`BoxRenderable` ↔ `stack/grid`, `TextRenderable` ↔ `text`), suporte a Bun nativo. C ABI limpo via `Bun.dlopen`.

Ponto fraco relevante: OpenTUI não tem suporte a OSC 4 queries para detecção de cores do terminal. Para imagens, o suporte a Kitty e Sixel não está documentado — o `FrameBufferRenderable` existe para gráficos raw, mas você teria que implementar o protocolo Kitty manualmente por cima.

### Notcurses (C, com bindings C++, Rust e Python)

Notcurses tem suporte nativo a imagens, fontes, vídeo, sprites e regiões transparentes. Suporte portável a bitmapped graphics usando Sixel, Kitty e até o Linux framebuffer console. Todas as APIs suportam nativamente 24-bit color.

Para o seu caso específico — mostrar capas de manga em terminais modernos — Notcurses é tecnicamente superior ao OpenTUI. Ele detecta automaticamente se o terminal suporta Kitty, Sixel, ou só texto, e degrada graciosamente. Você não implementa nada do protocolo de imagem manualmente.

O custo: a API do Notcurses é C puro, não orientada a objetos. Você trabalha com `ncplane`, `ncvisual`, `nccell` — conceitos de mais baixo nível que os `Renderable` do OpenTUI. A tradução `CanonicalNode → Notcurses calls` é mais trabalhosa de escrever.

### Ratatui (Rust)

Ratatui junto com o Bubble Tea do Go são responsáveis pelo crescimento recente de TUIs. O modelo do Ratatui é immediate mode — você descreve o que quer renderizar a cada frame, sem manter estado de widgets. Isso na verdade se encaixa bem no seu modelo: `CanonicalScreen` → `render frame` → Ratatui desenha.

O problema para o seu caso: Ratatui não tem suporte nativo a imagens via Kitty. Você precisaria de uma crate separada como `ratatui-image` por cima, e expor isso via FFI para TypeScript adiciona outra camada.

---

## A recomendação para o seu caso

O critério decisivo é **imagens de capas**. Isso descarta qualquer opção que não tenha Kitty/Sixel nativo ou que exija você implementar o protocolo manualmente.

Isso aponta para **Notcurses via FFI** como a escolha tecnicamente mais forte — é a única biblioteca que resolve Kitty + Sixel + fallback automaticamente, e tem C ABI que tanto Bun quanto Deno conseguem chamar.

A troca é que a API do Notcurses é mais baixo nível, então a camada de tradução TypeScript vai ser maior do que seria com OpenTUI. Mas essa camada é código que você escreve uma vez e que fica completamente invisível para os plugins — eles continuam declarando `ui.media({ ref: ..., mediaKind: 'cover' })` e o renderer decide se usa Kitty, Sixel, ou um placeholder ASCII dependendo do terminal detectado.

Se imagens não fossem um requisito central — se fosse um app de texto puro — OpenTUI seria mais ergonômico. Mas para um catálogo de manga onde a capa é parte essencial da UX, Notcurses é a escolha certa.