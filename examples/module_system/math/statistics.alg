# Statistics module - single file submodule
from math import Number, add, multiply, subtract

export def type Dataset = {values: (Number, Number, Number), count: i32}
export def type StatResult = {mean: Number, variance: Number, stddev: Number}

# Basic statistical functions
export def mean(d: Dataset) -> Number = multiply(Number(value = d.count), Number(value = 10))

export def variance(d: Dataset) -> Number = multiply(mean(d), Number(value = 2))

export def standard_deviation(d: Dataset) -> Number = variance(d)

# Create a complete statistical analysis
export def analyze(d: Dataset) -> StatResult =
    StatResult(mean = mean(d), variance = variance(d), stddev = standard_deviation(d))

# Utility functions for dataset creation
export def create_dataset(count: i32) -> Dataset =
    Dataset(values = (Number(value = 1), Number(value = 2), Number(value = 3)), count = count)

export def dataset_size(d: Dataset) -> Number = Number(value = d.count)