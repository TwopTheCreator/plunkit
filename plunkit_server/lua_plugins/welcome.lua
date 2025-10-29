function on_enable()
    plunkit.log("Welcome plugin enabled!")
end

function on_disable()
    plunkit.log("Welcome plugin disabled!")
end

function on_player_join(player_name)
    plunkit.log("Player joined: " .. player_name)

    plunkit.broadcast("Welcome to the server, " .. player_name .. "!")
    plunkit.broadcast("This server is powered by Plunkit WASM technology")

    return false
end

function on_player_chat(player_name, message)
    if message == "/help" then
        plunkit.broadcast("Available commands: /help, /spawn, /time")
        return true
    end

    return false
end
