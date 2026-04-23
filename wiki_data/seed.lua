local data = {
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Buckflower", count = 1 } },
    products = { { item = "Buckflower Seed", count = 2 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Citrome", count = 1 } },
    products = { { item = "Citrome Seed", count = 2 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Sandleaf", count = 1 } },
    products = { { item = "Sandleaf Seed", count = 2 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Aketine", count = 1 } },
    products = { { item = "Aketine Seed", count = 2 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Jincao", count = 1 } },
    products = { { item = "Jincao Seed", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Yazhen", count = 1 } },
    products = { { item = "Yazhen Seed", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Reed Rye", count = 1 } },
    products = { { item = "Reed Rye Seed", count = 2 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Tartpepper", count = 1 } },
    products = { { item = "Tartpepper Seed", count = 2 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Redjade Ginseng", count = 1 } },
    products = { { item = "Redjade Ginseng Seed", count = 2 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Amber Rice", count = 1 } },
    products = { { item = "Amber Rice Seed", count = 2 } },
},
}

for _, v in ipairs(data) do
    v.facility = "Seed-Picking Unit"
end

return data