public class NullLocal<T> { public T v; public void put(Object x) { Object y=null; this.v=(T)y; } }
