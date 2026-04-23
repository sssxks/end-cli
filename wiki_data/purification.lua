local data = {
{
    time = 2,
    mode = "Fluid",
    event = false,
    eventName = "",
    ingredients = { { item = "Inert Xircon Effluent", count = 4 } },
    products = { { item = "Xircon Effluent", count = 1 }, { item = "Clean Water", count = 1 } },
},
{
    time = 2,
    mode = "Fluid",
    event = false,
    eventName = "",
    ingredients = { { item = "Cuprium Solution", count = 4 } },
    products = { { item = "Hetonite Solution", count = 1 }, { item = "Precipitation Acid", count = 1 } },
},
}

for _, v in ipairs(data) do
    v.facility = "Purification Unit"
end

return data