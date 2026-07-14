# Para o seu caso de uso — catálogo de manga, grid de capas, listas, navegação por teclado — o subset mínimo real é um pouco maior:

// Ciclo de vida
notcurses_init
notcurses_stop
notcurses_render          ← renderiza tudo para o terminal

// Dimensões do terminal
notcurses_stddim_yx       ← largura/altura atual em células — ESSENCIAL
                          // você precisa disso para calcular layout

// Planos (sua unidade de composição)
ncplane_create
ncplane_destroy
ncplane_move_yx
ncplane_resize
ncplane_erase             ← limpar um plano antes de redesenhar — você vai precisar
ncplane_set_scrolling     ← para listas longas

// Texto
ncplane_putstr_yx
ncplane_putstr_aligned    ← para alinhar texto (center, right) sem calcular manualmente
ncplane_set_fg_rgb8
ncplane_set_bg_rgb8
ncplane_set_styles        ← bold, italic, underline

// Geometria de pixels — ESSENCIAL para imagens
ncplane_pixel_geom        ← quantos pixels tem uma célula no terminal atual
                          // sem isso você não sabe o tamanho real de uma imagem em células

// Imagens
ncvisual_from_file
ncvisual_from_memory      ← para imagens que você já tem em buffer (de rede, AT blob)
ncvisual_blit
ncvisual_destroy
ncvisual_geom             ← dimensões da imagem antes de renderizar

// Input
notcurses_get             ← eventos de teclado e mouse
notcurses_mice_enable     ← habilitar mouse — sem isso clique não funciona

// Capabilities
notcurses_capabilities    ← struct com tudo: suporte a cores, kitty, sixel, etc.
notcurses_canpixel        ← atalho: esse terminal suporta pixel graphics?

// Bordas e linhas
ncplane_box               ← bordas de painéis
ncplane_hline
ncplane_vline

## Simgle Thread Renderer