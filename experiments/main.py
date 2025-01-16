from time import perf_counter
import lifelib

sess = lifelib.load_rules('b3s23')
lt = sess.lifetree(n_layers=1, memory=8000)
with open('res/0e0p-metaglider.mc') as f:
    x = lt.pattern(f.read())

n = 2**23
t1 = perf_counter()
y = x[n]
t2 = perf_counter()
print(t2 - t1)
print(x.population, y.population)
