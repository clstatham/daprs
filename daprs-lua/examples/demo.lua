local daprs = require("daprs")

local graph = daprs.graph_builder()

local out1 = graph:output()
local out2 = graph:output()

local sine = graph:sine_osc()

sine:input("frequency"):set(440.0)

sine:output(0):connect(out1:input(0))
sine:output(0):connect(out2:input(0))

local runtime = graph:build_runtime()

runtime:run_for(1.0)

sine:input("frequency"):set(880.0)

runtime = graph:build_runtime()

runtime:run_for(1.0)
