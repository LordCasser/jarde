public class CycleRelay<T> { public T left(T x, boolean again) { if (again) return right(x,false); return x; } public T right(T x, boolean again) { if (again) return left(x,false); return x; } }
