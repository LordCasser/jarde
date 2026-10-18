import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Path;
import java.nio.file.Paths;

/** Parentless bounded reference driver for the generated classfile controls. */
public final class EntrySplitOracleDriver {
    private EntrySplitOracleDriver() {}
    public static void main(String[] args) throws Exception {
        if(args.length!=2)throw new IllegalArgumentException("usage: EntrySplitOracleDriver <actual-root> <expected-root>");
        Path actual=Paths.get(args[0]).toRealPath(), expected=Paths.get(args[1]).toRealPath();
        try(URLClassLoader loader=new URLClassLoader(new URL[]{actual.toUri().toURL()},null)){
            Class<?> c=Class.forName("cf08.EntrySplitCases",true,loader);
            if(c.getClassLoader()!=loader)throw new AssertionError("target loader mismatch");
            Path source=Paths.get(c.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
            if(!source.equals(expected))throw new IllegalStateException("wrong CodeSource: "+source+" expected "+expected);
            Field[] fields=c.getDeclaredFields();
            Constructor<?>[] constructors=c.getDeclaredConstructors();
            Method[] methods=c.getDeclaredMethods();
            if(fields.length!=0||constructors.length!=1||methods.length!=4)
                throw new AssertionError("target inventory differs: fields="+fields.length+" constructors="+constructors.length+" methods="+methods.length);
            if(constructors[0].getParameterTypes().length!=0||!Modifier.isPublic(constructors[0].getModifiers()))
                throw new AssertionError("expected one public no-argument constructor");
            verifyInventoryOnly(actual, expected, "cf08.EntrySplitRefusals", new String[]{"booleanPrefix(ZZI)I", "bytePrefix(BZI)I", "alias(IZ)I", "callPrefix(IZI)I"});
            verifyInventoryOnly(actual, expected, "cf08.EntrySplitTopologyRefusals", new String[]{"extraEntry(IZI)I", "extraExit(IZI)I", "extraLive(IZZI)I", "stackEscape(IZI)I"});
            verifyInventoryOnly(actual, expected, "cf08.EntrySplitExceptionRefusal", new String[]{"exceptionPrefix(IZII)I"});
            verifyInventoryOnly(actual, expected, "cf08.EntrySplitLegacyCases", new String[]{"legacy(IZI)I"});
            for(String name:new String[]{"test","signed","shifted","inverted"}){
                Method method=null;for(Method m:methods)if(m.getName().equals(name))method=m;
                if(method==null||!Modifier.isPublic(method.getModifiers())||!Modifier.isStatic(method.getModifiers())||method.getReturnType()!=int.class)
                    throw new AssertionError("missing or malformed "+name);
                Class<?>[] expectedParameters=name.equals("shifted")
                    ?new Class<?>[]{int.class,int.class,int.class,boolean.class,int.class}
                    :new Class<?>[]{int.class,boolean.class,int.class};
                if(!java.util.Arrays.equals(method.getParameterTypes(),expectedParameters))
                    throw new AssertionError("wrong descriptor for "+name);
                for(int initial:new int[]{17,Integer.MAX_VALUE-1,Integer.MIN_VALUE+1})for(boolean skip:new boolean[]{false,true})for(int steps=0;steps<=2;steps++){
                    Object[] arguments=method.getParameterTypes().length==5
                        ?new Object[]{initial,0,0,skip,steps}:new Object[]{initial,skip,steps};
                    int actualValue;
                    try{actualValue=((Integer)method.invoke(null,arguments)).intValue();}
                    catch(InvocationTargetException e){Throwable t=e.getCause();if(t instanceof Exception)throw(Exception)t;if(t instanceof Error)throw(Error)t;throw new AssertionError(t);}
                    int amount=name.equals("signed")?-5:name.equals("inverted")?-2:name.equals("shifted")?5:3;
                    int expectedValue=initial;if(!skip)expectedValue+=amount;int remaining=steps;while(remaining!=0){remaining--;expectedValue+=amount;}
                    String label=name+"-initial-"+initial+"-skip-"+skip+"-steps-"+steps;
                    System.out.println("CASE name="+label+" actual="+actualValue+" expected="+expectedValue);
                    if(actualValue!=expectedValue)throw new AssertionError(label+" differs from integer source oracle");
                }
            }
        }
    }
    private static String descriptor(Method method) {
        StringBuilder out=new StringBuilder("(");
        for(Class<?> parameter:method.getParameterTypes())out.append(descriptorType(parameter));
        return out.append(')').append(descriptorType(method.getReturnType())).toString();
    }
    private static String descriptorType(Class<?> type) {
        if(type==int.class)return "I";if(type==boolean.class)return "Z";if(type==byte.class)return "B";
        if(type==void.class)return "V";if(type.isArray())return type.getName().replace('.', '/');
        return "L"+type.getName().replace('.', '/')+";";
    }
    private static void verifyInventoryOnly(Path actual, Path expected, String className, String[] names) throws Exception {
        try(URLClassLoader loader=new URLClassLoader(new URL[]{actual.toUri().toURL()},null)){
            Class<?> c=Class.forName(className,true,loader);
            if(c.getClassLoader()!=loader)throw new AssertionError("target loader mismatch for "+className);
            Path source=Paths.get(c.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
            if(!source.equals(expected))throw new IllegalStateException("wrong CodeSource: "+source+" expected "+expected);
            if(c.getDeclaredFields().length!=0||c.getDeclaredConstructors().length!=1||c.getDeclaredMethods().length!=names.length)
                throw new AssertionError("unexpected inventory for "+className);
            for(String signature:names){
                int split=signature.indexOf('(');String name=signature.substring(0,split),descriptor=signature.substring(split);
                Method found=null;for(Method method:c.getDeclaredMethods())if(method.getName().equals(name))found=method;
                if(found==null||!Modifier.isPublic(found.getModifiers())||!Modifier.isStatic(found.getModifiers())||found.getReturnType()!=int.class)
                    throw new AssertionError("missing public static int control "+signature);
                String actualDescriptor=descriptor(found);
                if(!actualDescriptor.equals(descriptor))throw new AssertionError("descriptor "+actualDescriptor+" != "+descriptor);
            }
            Constructor<?> constructor=c.getDeclaredConstructors()[0];
            if(c.getDeclaredFields().length!=0||c.getDeclaredConstructors().length!=1||!Modifier.isPublic(constructor.getModifiers())||constructor.getParameterTypes().length!=0)
                throw new AssertionError("constructor/field inventory differs for "+className);
            constructor.newInstance();
            System.out.println("INVENTORY class="+className+" methods="+names.length+" methodsInvoked=false");
        }
    }

}
