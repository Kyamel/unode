unode AST node          →  OpenTUI renderable(s)
─────────────────────────────────────────────────
stack / inline / grid   →  BoxRenderable (flexDirection varia)
text                    →  TextRenderable
value                   →  TextRenderable (com formatação aplicada)
media (com Kitty)       →  FrameBufferRenderable
input                   →  InputRenderable
input (select)          →  SelectRenderable
disclosure              →  BoxRenderable + TextRenderable (toggle)
list                    →  BoxRenderable + N BoxRenderable filhos
action                  →  BoxRenderable focusável + onMouseDown + onKeyDown
scroll                  →  ScrollBoxRenderable
loading                 →  TextRenderable animado (live: true)
conditional             →  visible = true/false no renderable
slot                    →  BoxRenderable vazio que recebe .add() de contribuições