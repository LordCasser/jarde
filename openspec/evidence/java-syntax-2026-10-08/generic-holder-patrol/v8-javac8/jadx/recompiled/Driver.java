package defpackage;
public class Driver {
    public static void main(String[] args) throws Exception {
        Hold<String> h=new Hold<>("ok"); System.out.println("value="+h.v);
        System.out.println("classvars="+Hold.class.getTypeParameters().length);
        System.out.println("field="+Hold.class.getField("v").getGenericType().getTypeName());
        System.out.println("ctor="+Hold.class.getConstructor(Object.class).getGenericParameterTypes()[0].getTypeName());
    }
}
