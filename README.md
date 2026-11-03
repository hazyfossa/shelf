# shelf

Shelf is a simple shell-config manager. It is different from a plugin manager (ohmyzsh/fisher/etc) in that it does not manage software. Instead, `shelf` applies fixed code snippets based on what executables you have in PATH.

When you install software that warrants shell integration (starship, zoxide, etc), `shelf` will manage the required shell code. That way, your config remains portable across machines with different software. And, as a bonus, you won't need to remember deleting some stuff after `pacman -Rns`

A (not yet accomplished) goal of `shelf` is to be portable. This means that it should be reasonably easy to write a `.shelf` config that works the same across bash/zsh/fish/etc.

This project was written for me to:
- replace the horrible mess of chezmoi rules that is my config.fish
- learn to write chumsky parsers