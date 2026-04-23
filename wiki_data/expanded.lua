local data = {
{
    time = 2,
    mode = "Fluid",
    event = false,
    eventName = "",
    ingredients = { { item = "Jincao Powder", count = 1 }, { item = "Clean Water", count = 1 } },
    products = { { item = "Jincao Solution", count = 1 } },
},
{
    time = 2,
    mode = "Fluid",
    event = false,
    eventName = "",
    ingredients = { { item = "Yazhen Powder", count = 1 }, { item = "Clean Water", count = 1 } },
    products = { { item = "Yazhen Solution", count = 1 } },
},
{
    time = 2,
    mode = "Fluid",
    event = false,
    eventName = "",
    ingredients = { { item = "Xiranite", count = 1 }, { item = "Clean Water", count = 1 } },
    products = { { item = "Liquid Xiranite", count = 1 } },
},
{
    time = 2,
    mode = "Fluid",
    event = false,
    eventName = "",
    ingredients = { { item = "Heavy Xiranite", count = 1 }, { item = "Precipitation Acid", count = 1 } },
    products = { { item = "Liquid Heavy Xiranite", count = 1 } },
},
{
    time = 2,
    mode = "Fluid",
    event = false,
    eventName = "",
    ingredients = { { item = "Cuprium Powder", count = 1 }, { item = "Precipitation Acid", count = 1 } },
    products = { { item = "Cuprium Solution", count = 1 } },
},
{
    time = 2,
    mode = "Fluid",
    event = false,
    eventName = "",
    ingredients = { { item = "Liquid Xiranite", count = 1 }, { item = "Sewage", count = 1 } },
    products = { { item = "Xircon Effluent", count = 1 }, { item = "Inert Xircon Effluent", count = 1 } },
},
{
    time = 2,
    mode = "Fluid",
    event = false,
    eventName = "",
    ingredients = { { item = "Xircon Effluent", count = 2 }, { item = "Ferrium Powder", count = 1 } },
    products = { { item = "Xircon", count = 1 }, { item = "Sewage", count = 1 } },
},
{
    time = 2,
    mode = "Fluid",
    event = false,
    eventName = "",
    ingredients = { { item = "Hetonite Solution", count = 2 }, { item = "Ferrium Powder", count = 1 } },
    products = { { item = "Hetonite", count = 1 }, { item = "Sewage", count = 1 } },
},
}

for _, v in ipairs(data) do
    v.facility = "Expanded Crucible"
end

return data