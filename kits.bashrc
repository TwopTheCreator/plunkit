if [ -d "$PWD/scripts" ]; then
    export PATH="$PWD/scripts:$PATH"
fi

PS1="\[\e[32m\]\u@\h\[\e[m\]:\[\e[34m\]\w\[\e[m\]\$ "

alias ll='ls -la'
alias gs='git status -sb'
alias gc='git commit -m'
alias gp='git push origin $(git branch --show-current)'
alias gpl='git pull origin $(git branch --show-current)'

alias deploy='/git/deploy.sh'

runjar() {
    if [ -z "$1" ]; then
        echo "Usage: runjar <file.jar>"
        return 1
    fi

    if [ ! -f "$1" ]; then
        echo "⚠️ File $1 does not exist!"
        return 1
    fi

    echo "☕ Running $1 with Java..."
    java -jar "$1"
}

plunkit-run() {
    local jarfile="$1"

    if [ -z "$jarfile" ]; then
        jarfile=$(ls *.jar 2>/dev/null | head -n1)
        if [ -z "$jarfile" ]; then
            echo "No .jar file found in current directory!"
            return 1
        fi
    fi

    echo "🚀 Initializing Plunkit kit: $jarfile"
    java -Xmx2G -jar "$jarfile"
}

if [ -f /usr/share/bash-completion/completions/git ]; then
    . /usr/share/bash-completion/completions/git
fi

export PLUNKIT_ENV="development"
export PLUNKIT_PATH="$PWD"
