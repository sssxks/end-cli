local data = {
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Ferrium", count = 2 } },
    products = { { item = "Ferrium Bottle", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Amethyst Fiber", count = 2 } },
    products = { { item = "Amethyst Bottle", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Steel", count = 2 } },
    products = { { item = "Steel Bottle", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Cryston Fiber", count = 2 } },
    products = { { item = "Cryston Bottle", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Cuprium", count = 2 } },
    products = { { item = "Cuprium Bottle", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Hetonite", count = 2 } },
    products = { { item = "Hetonite Bottle", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = true,
    eventName = "Palm-Top Savior",
    ingredients = { { item = "Xiranite", count = 2 } },
    products = { { item = "Experimental Xiranite Bottle", count = 1 } },
},
{
    time = 20,
    mode = "Solid",
    event = true,
    eventName = "Palm-Top Savior",
    ingredients = { { item = "Heavy Xiranite", count = 1 } },
    products = { { item = "Experimental Heavy Xiranite Bottle", count = 2 } },
},
}

for _, v in ipairs(data) do
    v.facility = "Moulding Unit"
end

return data