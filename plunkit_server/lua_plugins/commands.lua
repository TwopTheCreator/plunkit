function on_enable()
    plunkit.log("Commands plugin enabled!")
end

function on_disable()
    plunkit.log("Commands plugin disabled!")
end

function on_player_chat(player_name, message)
    if message == "/help" then
        plunkit.broadcast("=== Available Commands ===")
        plunkit.broadcast("/help - Show this help message")
        plunkit.broadcast("/spawn - Teleport to spawn")
        plunkit.broadcast("/players - List online players")
        plunkit.broadcast("/time - Show server time")
        return true
    end

    if message == "/spawn" then
        plunkit.teleport_player(player_name, 0, 64, 0)
        plunkit.broadcast(player_name .. " teleported to spawn")
        return true
    end

    if message == "/players" then
        local players = plunkit.get_players()
        local count = 0
        for _ in pairs(players) do count = count + 1 end

        plunkit.broadcast("Online players (" .. count .. "):")
        for _, p in pairs(players) do
            plunkit.broadcast("  - " .. p)
        end
        return true
    end

    if message == "/time" then
        plunkit.broadcast("Server has been running for a while!")
        return true
    end

    if string.sub(message, 1, 5) == "/give" then
        local args = {}
        for word in string.gmatch(message, "%S+") do
            table.insert(args, word)
        end

        if #args >= 3 then
            local target = args[2]
            local item = args[3]
            local count = tonumber(args[4]) or 1

            plunkit.give_item(target, item, count)
            plunkit.broadcast("Gave " .. count .. " " .. item .. " to " .. target)
            return true
        end
    end

    return false
end
