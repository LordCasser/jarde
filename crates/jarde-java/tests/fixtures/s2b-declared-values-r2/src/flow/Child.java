package flow;
public class Child extends Mid<identity.Foo> {
    public identity.Foo knownFlow(java.util.List<identity.Foo> input, java.util.List<identity.Foo>[] arrays, boolean choose, int index) {
        java.util.List<identity.Foo> local;
        if (choose) local = input; else local = arrays[index];
        java.util.List<identity.Foo> copy = local;
        return copy.get(index);
    }
    public Object unknownFlow(java.util.List<identity.Foo> known, java.util.List[] arrays, boolean choose, int index) {
        java.util.List local;
        if (choose) local = known; else local = arrays[index];
        return local.get(index);
    }
    public Object nullOnly(int index) {
        java.util.List<identity.Foo> local = null;
        return local.get(index);
    }
    public Object inheritedField() { return field; }
    public Object inheritedMethod() { return getBase(); }
}
