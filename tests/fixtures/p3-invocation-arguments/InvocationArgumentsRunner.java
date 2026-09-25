public class InvocationArgumentsRunner {
    public static void main(String[] args) {
        System.out.println("objectString=" + InvocationArgumentsProbe.objectString());
        System.out.println("objectNull=" + InvocationArgumentsProbe.objectNull());
        System.out.println("objectArray=" + InvocationArgumentsProbe.objectArray(new String[] {"x"}));
        System.out.println("objectBoxed=" + InvocationArgumentsProbe.objectBoxed());
        System.out.println("widening=" + InvocationArgumentsProbe.widening('A'));
        System.out.println("narrowByte=" + InvocationArgumentsProbe.narrowByte());
        System.out.println("narrowShort=" + InvocationArgumentsProbe.narrowShort());
        System.out.println("constructorObject=" + InvocationArgumentsProbe.constructorObject());
        System.out.println("multiple=" + InvocationArgumentsProbe.multiple('A'));
        System.out.println("genericObject=" + InvocationArgumentsProbe.genericObject());
        System.out.println("functionalRunnable=" + InvocationArgumentsProbe.functionalRunnable());
        System.out.println("functionalObject=" + InvocationArgumentsProbe.functionalObject());
    }
}
