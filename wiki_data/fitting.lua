local data = {
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Ferrium", count = 1 } },
    products = { { item = "Ferrium Part", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Amethyst Fiber", count = 1 } },
    products = { { item = "Amethyst Part", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Steel", count = 1 } },
    products = { { item = "Steel Part", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Cryston Fiber", count = 1 } },
    products = { { item = "Cryston Part", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Cuprium", count = 1 } },
    products = { { item = "Cuprium Part", count = 1 } },
},
{
    time = 10,
    mode = "Solid",
    event = false,
    eventName = "",
    ingredients = { { item = "Hetonite", count = 5 } },
    products = { { item = "Hetonite Part", count = 1 } },
},
{
    time = 2,
    mode = "Solid",
    event = true,
    eventName = "Palm-Top Savior",
    ingredients = { { item = "Xiranite", count = 1 } },
    products = { { item = "Experimental Xiranite Part", count = 1 } },
},
{
    time = 20,
    mode = "Solid",
    event = true,
    eventName = "Palm-Top Savior",
    ingredients = { { item = "Heavy Xiranite", count = 1 } },
    products = { { item = "Experimental Heavy Xiranite Part", count = 2 } },
},
}

for _, v in ipairs(data) do
    v.facility = "Fitting Unit"
end

return data