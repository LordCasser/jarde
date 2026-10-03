class Box<T> {
    void set(T v) { System.out.println("BOX.set(Object) ran"); }
    T get() { System.out.println("BOX.get ran"); return null; }
}
