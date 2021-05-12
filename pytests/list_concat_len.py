list1 = [1,2]
list2 = [3,4]
list3 = list1 + list2
list_len = len(list3)

if list_len != 4:
    panic("List len() is " + str(list_len)+ " instead of 4")