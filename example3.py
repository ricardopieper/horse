def double(x):
    return x * 2

sum = 0
current = 1
while current < 10:
    val = double(current)
    print(val)
    sum = sum + val
    current = current + 1

print("sum should be 90, is " + str(sum))

proof = 2 + 4 + 6 + 8 + 10 + 12 + 14 + 16 + 18
print("proof: "+str(proof))