local save_interval = 300
local last_save = 0

function on_enable()
    plunkit.log("Autosave plugin enabled! Saving every " .. save_interval .. " seconds")
    last_save = 0
end

function on_disable()
    plunkit.log("Autosave plugin disabled!")
end

function tick()
    last_save = last_save + 1

    if last_save >= save_interval * 20 then
        plunkit.log("Auto-saving world...")
        plunkit.broadcast("Saving world data...")

        last_save = 0
    end
end
