interface SupG<T> { T get(); }
class CovGen implements SupG {
    public String get() { return "s"; }
}
