local player_join_count = 0
local player_blocks_broken = {}
local player_blocks_placed = {}

function on_enable()
    plunkit.log("Events tracker plugin enabled!")
end

function on_disable()
    plunkit.log("Events tracker plugin disabled!")
    plunkit.log("Total joins: " .. player_join_count)
end

function on_player_join(player_name)
    player_join_count = player_join_count + 1

    if not player_blocks_broken[player_name] then
        player_blocks_broken[player_name] = 0
    end
    if not player_blocks_placed[player_name] then
        player_blocks_placed[player_name] = 0
    end

    plunkit.log("Total player joins: " .. player_join_count)

    if player_blocks_broken[player_name] > 0 or player_blocks_placed[player_name] > 0 then
        plunkit.broadcast(player_name .. " stats: " ..
            player_blocks_broken[player_name] .. " blocks broken, " ..
            player_blocks_placed[player_name] .. " blocks placed")
    end

    return false
end

function on_block_break(player_name, x, y, z)
    if not player_blocks_broken[player_name] then
        player_blocks_broken[player_name] = 0
    end

    player_blocks_broken[player_name] = player_blocks_broken[player_name] + 1

    if player_blocks_broken[player_name] % 100 == 0 then
        plunkit.broadcast(player_name .. " has broken " .. player_blocks_broken[player_name] .. " blocks!")
    end

    return false
end

function on_block_place(player_name, x, y, z, block_id)
    if not player_blocks_placed[player_name] then
        player_blocks_placed[player_name] = 0
    end

    player_blocks_placed[player_name] = player_blocks_placed[player_name] + 1

    if player_blocks_placed[player_name] % 100 == 0 then
        plunkit.broadcast(player_name .. " has placed " .. player_blocks_placed[player_name] .. " blocks!")
    end

    return false
end
