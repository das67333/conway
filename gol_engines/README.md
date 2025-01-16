# Explanation (TODO)

## Hashlife algorithm

#### First pipeline

Update 4Nx4N by N steps using 13 2Nx2N updates by N/2 steps.

1) Decay 1 4Nx4N to 9 2Nx2N
`fn nine_children_overlapping`
$$
\begin{bmatrix}
a_{11} & a_{12} & a_{13} & a_{14} \\
a_{21} & a_{22} & a_{23} & a_{24} \\
a_{31} & a_{32} & a_{33} & a_{34} \\
a_{41} & a_{42} & a_{43} & a_{44} \\
\end{bmatrix}
\implies
\begin{bmatrix}
a_{11} & a_{12} \\
a_{21} & a_{22} \\
\end{bmatrix}
\begin{bmatrix}
a_{12} & a_{13} \\
a_{22} & a_{23} \\
\end{bmatrix}
\begin{bmatrix}
a_{13} & a_{14} \\
a_{23} & a_{24} \\
\end{bmatrix}
\begin{bmatrix}
a_{21} & a_{22} \\
a_{31} & a_{32} \\
\end{bmatrix}
\begin{bmatrix}
a_{22} & a_{23} \\
a_{32} & a_{33} \\
\end{bmatrix}
\begin{bmatrix}
a_{23} & a_{24} \\
a_{33} & a_{34} \\
\end{bmatrix}
\begin{bmatrix}
a_{31} & a_{32} \\
a_{41} & a_{42} \\
\end{bmatrix}
\begin{bmatrix}
a_{32} & a_{33} \\
a_{42} & a_{43} \\
\end{bmatrix}
\begin{bmatrix}
a_{33} & a_{34} \\
a_{43} & a_{44} \\
\end{bmatrix}
$$

2) Update 9 2Nx2N, getting 9 1Nx1N

3) Combine 9 1Nx1N to 4 2Nx2N
`fn four_children_overlapping`
$$
\begin{bmatrix}
a_{11} & a_{12} & a_{13} \\
a_{21} & a_{22} & a_{23} \\
a_{31} & a_{32} & a_{33} \\
\end{bmatrix}
\implies
\begin{bmatrix}
a_{11} & a_{12} \\
a_{21} & a_{22} \\
\end{bmatrix}
\begin{bmatrix}
a_{12} & a_{13} \\
a_{22} & a_{23} \\
\end{bmatrix}
\begin{bmatrix}
a_{21} & a_{22} \\
a_{31} & a_{32} \\
\end{bmatrix}
\begin{bmatrix}
a_{22} & a_{23} \\
a_{32} & a_{33} \\
\end{bmatrix}
$$

4) Update 4 2Nx2N, getting 4 1Nx1N and uniting them to 1 2Nx2N


#### Second pipeline

Same as before but skipping first group of updates.
Update 4Nx4N by k steps using 4 2Nx2N updates by k steps.

First step is replaced with
`fn nine_children_disjoint`
$$
\begin{bmatrix}
a_{11} & a_{12} & a_{13} & a_{14} & a_{15} & a_{16} & a_{17} & a_{18} \\
a_{21} & a_{22} & a_{23} & a_{24} & a_{25} & a_{26} & a_{27} & a_{28} \\
a_{31} & a_{32} & a_{33} & a_{34} & a_{35} & a_{36} & a_{37} & a_{38} \\
a_{41} & a_{42} & a_{43} & a_{44} & a_{45} & a_{46} & a_{47} & a_{48} \\
a_{51} & a_{52} & a_{53} & a_{54} & a_{55} & a_{56} & a_{57} & a_{58} \\
a_{61} & a_{62} & a_{63} & a_{64} & a_{65} & a_{66} & a_{67} & a_{68} \\
a_{71} & a_{72} & a_{73} & a_{74} & a_{75} & a_{76} & a_{77} & a_{78} \\
a_{81} & a_{82} & a_{83} & a_{84} & a_{85} & a_{86} & a_{87} & a_{88} \\
\end{bmatrix}
\implies
\begin{bmatrix}
a_{11} & a_{12} \\
a_{21} & a_{22} \\
\end{bmatrix}
\begin{bmatrix}
a_{13} & a_{14} \\
a_{23} & a_{24} \\
\end{bmatrix}
\begin{bmatrix}
a_{15} & a_{16} \\
a_{25} & a_{26} \\
\end{bmatrix}
\begin{bmatrix}
a_{31} & a_{32} \\
a_{41} & a_{42} \\
\end{bmatrix}
\begin{bmatrix}
a_{33} & a_{34} \\
a_{43} & a_{44} \\
\end{bmatrix}
\begin{bmatrix}
a_{35} & a_{36} \\
a_{45} & a_{46} \\
\end{bmatrix}
\begin{bmatrix}
a_{51} & a_{52} \\
a_{61} & a_{62} \\
\end{bmatrix}
\begin{bmatrix}
a_{53} & a_{54} \\
a_{63} & a_{64} \\
\end{bmatrix}
\begin{bmatrix}
a_{55} & a_{56} \\
a_{65} & a_{66} \\
\end{bmatrix}
$$