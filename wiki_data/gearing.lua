local data = {
{
    time = 10,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Origocrust", count = 5 }, { item = "Amethyst Fiber", count = 5 } },
    products = { { item = "Amethyst Component", count = 1 } },
},
{
    time = 10,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Origocrust", count = 10 }, { item = "Ferrium", count = 10 } },
    products = { { item = "Ferrium Component", count = 1 } },
},
{
    time = 10,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Packed Origocrust", count = 10 }, { item = "Cryston Fiber", count = 10 } },
    products = { { item = "Cryston Component", count = 1 } },
},
{
    time = 10,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Hetonite Part", count = 2 }, { item = "Heavy Xiranite", count = 2 } },
    products = { { item = "Hetonite Component", count = 1 } },
},
{
    time = 10,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Cuprium Part", count = 10 }, { item = "Xiranite", count = 10 } },
    products = { { item = "Cuprium Component", count = 1 } },
},
{
    time = 10,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Packed Origocrust", count = 10 }, { item = "Xiranite", count = 10 } },
    products = { { item = "Xiranite Component", count = 1 } },
},
}

for _, v in ipairs(data) do
    v.facility = "Gearing Unit"
end

return data