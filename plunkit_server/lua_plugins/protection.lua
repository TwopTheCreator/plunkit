local protected_blocks = {}

function on_enable()
    plunkit.log("Protection plugin enabled!")

    protected_blocks[0] = true
    protected_blocks[1] = true
    protected_blocks[7] = true
end

function on_disable()
    plunkit.log("Protection plugin disabled!")
end

function on_block_break(player_name, x, y, z)
    local block_id = plunkit.get_block(x, y, z)

    if protected_blocks[block_id] then
        plunkit.log(player_name .. " tried to break protected block at " .. x .. ", " .. y .. ", " .. z)
        return true
    end

    plunkit.log(player_name .. " broke block at " .. x .. ", " .. y .. ", " .. z)
    return false
end

function on_block_place(player_name, x, y, z, block_id)
    if y > 256 then
        plunkit.log(player_name .. " tried to place block above height limit")
        return true
    end

    if y < 0 then
        plunkit.log(player_name .. " tried to place block below bedrock")
        return true
    end

    return false
end
