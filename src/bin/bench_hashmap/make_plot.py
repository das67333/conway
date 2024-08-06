import matplotlib.pyplot as plt

# with open('data.txt') as f:
#     data = list(f.readlines())

x, y1, y2 = [], [], []
for line in filter(bool, data):
    n, m, per_insert, per_find = map(float, line.split())
    x.append(n / m)
    y1.append(per_insert)
    y2.append(per_find)

plt.xlim(0, 1)
plt.ylim(0, 200)
plt.grid()
plt.plot(x, y1, label='ns per insert')
plt.plot(x, y2, label='ns per find')
plt.legend()
