local data = {
{
    time = 2,
    mode = "Fluid",
    event = false,
    eventName = "",
    ingredients = { { item = "Stabilized Carbon", count = 2 }, { item = "Clean Water", count = 1 } },
    products = { { item = "Xiranite", count = 1 } },
},
{
    time = 10,
    mode = "Fluid",
    event = false,
    eventName = "",
    ingredients = { { item = "Xiranite", count = 10 }, { item = "Xircon Effluent", count = 5 } },
    products = { { item = "Heavy Xiranite", count = 1 } },
},
{
    time = 2,
    mode = "Fluid",
    event = false,
    eventName = "",
    ingredients = { { item = "Burdo-Muck", count = 1 }, { item = "Liquid Xiranite", count = 1 } },
    products = { { item = "Bumper-Rich", count = 1 } },
},
}

for _, v in ipairs(data) do
    v.facility = "Forge of the Sky"
end

return data