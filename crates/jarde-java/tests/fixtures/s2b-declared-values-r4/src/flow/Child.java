package flow;
public class Child<T extends java.util.List<identity.Foo>> extends Mid<identity.Foo> {
    public identity.Foo fromGetter(int index) { return getBase().get(index); }
    public identity.Foo fromField(int index) { return field.get(index); }
    public Object unknownObject(Object value, int index) { return ((java.util.List) value).get(index); }
    public identity.Foo fromClassBound(T value, int index) { return value.get(index); }
    public <U extends java.util.List<identity.Foo>> identity.Foo fromMethodBound(U value, int index) { return value.get(index); }
}
