def compute(param, func):
    return func(param, 2)

def function_to_pass(param1, param2):
    return param1 * param2

result = compute(3, function_to_pass)

assert_eq(6, result)
