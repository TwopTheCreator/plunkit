if [ -d "$HOME/plunkit-scripts" ]; then
    export PATH="$HOME/plunkit-scripts:$PATH"
fi

# Aliases
alias gs='git status -sb'
alias gc='git commit -m'
alias gp='git push origin $(git branch --show-current)'
alias gpl='git pull origin $(git branch --show-current)'
alias deploy='./git/deploy.sh'

PS1="\[\e[32m\]\u@\h\[\e[m\]:\[\e[34m\]\w\[\e[m\]\$ "

if [ -f /usr/share/bash-completion/completions/git ]; then
    . /usr/share/bash-completion/completions/git
fi

echo "Plunkit bashrc loaded!"
