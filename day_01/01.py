#!/usr/bin/env python
# encoding: utf-8
#

if __name__ == "__main__":
    def get_numbers():
        with open("input_01.txt", "r") as f:
            yield from map(int, f.readlines())

    def get_fuel(n):
        intermediate = (n//3) - 2
        if intermediate > 0:
            return intermediate + get_fuel(intermediate)
        else:
            return 0

    for n in get_numbers():
        print(f"{n} --(//3)-> {n//3} --(-2)-> {n//3-2} "
              f"[get_fuel: {get_fuel(n)}]")

    print(sum(map(get_fuel, get_numbers())))
