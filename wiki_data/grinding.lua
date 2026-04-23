local data = {
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Ferrium Powder", count = 2 }, { item = "Sandleaf Powder", count = 1 } },
    products = { { item = "Dense Ferrium Powder", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Amethyst Powder", count = 2 }, { item = "Sandleaf Powder", count = 1 } },
    products = { { item = "Cryston Powder", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Originium Powder", count = 2 }, { item = "Sandleaf Powder", count = 1 } },
    products = { { item = "Dense Originium Powder", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Carbon Powder", count = 2 }, { item = "Sandleaf Powder", count = 1 } },
    products = { { item = "Dense Carbon Powder", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Origocrust Powder", count = 2 }, { item = "Sandleaf Powder", count = 1 } },
    products = { { item = "Dense Origocrust Powder", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Buckflower Powder", count = 2 }, { item = "Sandleaf Powder", count = 1 } },
    products = { { item = "Ground Buckflower Powder", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Citrome Powder", count = 2 }, { item = "Sandleaf Powder", count = 1 } },
    products = { { item = "Ground Citrome Powder", count = 1 } },
},
}

for _, v in ipairs(data) do
    v.facility = "Grinding Unit"
end

return data