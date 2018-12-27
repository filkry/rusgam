"set wildignore =*.rs
set wildignore +=*/target/*
set wildignore +=*.dll
set wildignore +=*.lib

"set makeprg="cargo build"
compiler! cargo
map <A-b> :make build<CR>
