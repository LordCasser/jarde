import java.lang.reflect.Constructor;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.Arrays;

/** Finite parentless oracle for the two generated slot/counter controls. */
public final class EntrySplitSlotCounterCasesDriver {
    private EntrySplitSlotCounterCasesDriver() {}
    public static void main(String[] args) throws Exception {
        if(args.length!=2) throw new IllegalArgumentException("usage: EntrySplitSlotCounterCasesDriver <actual-root> <expected-root>");
        Path actual=Paths.get(args[0]).toRealPath(), expected=Paths.get(args[1]).toRealPath();
        try(URLClassLoader loader=new URLClassLoader(new URL[]{actual.toUri().toURL()},null)) {
            Class<?> type=Class.forName("cf08.EntrySplitSlotCounterCases",true,loader);
            if(type.getClassLoader()!=loader) throw new AssertionError("target loader mismatch");
            Path source=Paths.get(type.getProtectionDomain().getCodeSource().getLocation().toURI()).toRealPath();
            if(!source.equals(expected)) throw new IllegalStateException("wrong CodeSource: "+source+" expected "+expected);
            if(type.getDeclaredFields().length!=0||type.getDeclaredMethods().length!=2||type.getDeclaredConstructors().length!=1)
                throw new AssertionError("unexpected target inventory");
            Constructor<?> constructor=type.getDeclaredConstructors()[0];
            if(!Modifier.isPublic(constructor.getModifiers())||constructor.getParameterTypes().length!=0)
                throw new AssertionError("expected public no-argument constructor");
            constructor.newInstance();
            checkMethod(type,"permuted",new Class<?>[]{boolean.class,int.class,int.class},"(ZII)I");
            checkMethod(type,"countUp",new Class<?>[]{int.class,boolean.class,int.class},"(IZI)I");
            Method permuted=type.getDeclaredMethod("permuted",boolean.class,int.class,int.class);
            Method countUp=type.getDeclaredMethod("countUp",int.class,boolean.class,int.class);
            int rows=0;
            for(int value:new int[]{17,Integer.MAX_VALUE-1,Integer.MIN_VALUE+1})
                for(boolean skip:new boolean[]{false,true})
                    for(int counter=0;counter<=2;counter++) {
                        int expectedValue=value;
                        if(!skip) expectedValue+=7;
                        int remaining=counter;
                        while(remaining!=0) { remaining--; expectedValue+=7; }
                        emit(permuted,new Object[]{skip,counter,value},"permuted",skip,value,counter,expectedValue);
                        rows++;
                    }
            for(int value:new int[]{17,Integer.MAX_VALUE-1,Integer.MIN_VALUE+1})
                for(boolean skip:new boolean[]{false,true})
                    for(int counter=0;counter>=-2;counter--) {
                        int expectedValue=value;
                        if(!skip) expectedValue+=3;
                        int remaining=counter;
                        while(remaining!=0) { remaining++; expectedValue+=3; }
                        emit(countUp,new Object[]{value,skip,counter},"countUp",skip,value,counter,expectedValue);
                        rows++;
                    }
            if(rows!=36) throw new AssertionError("expected 36 finite rows, got "+rows);
        }
    }
    private static void checkMethod(Class<?> owner,String name,Class<?>[] parameters,String descriptor) throws Exception {
        Method method=owner.getDeclaredMethod(name,parameters);
        if(!Modifier.isPublic(method.getModifiers())||!Modifier.isStatic(method.getModifiers())||method.getReturnType()!=int.class)
            throw new AssertionError("malformed method "+name);
        String actual="(";
        for(Class<?> parameter:method.getParameterTypes()) actual+=descriptorType(parameter);
        actual+=")"+descriptorType(method.getReturnType());
        if(!Arrays.equals(actual.toCharArray(),descriptor.toCharArray()))
            throw new AssertionError("descriptor for "+name+": "+actual+" != "+descriptor);
    }
    private static String descriptorType(Class<?> type) {
        if(type==int.class)return "I"; if(type==boolean.class)return "Z"; if(type==void.class)return "V";
        if(type.isArray())return type.getName().replace('.','/');
        return "L"+type.getName().replace('.','/')+";";
    }
    private static void emit(Method method,Object[] arguments,String name,boolean skip,int value,int counter,int expected) throws Exception {
        int actual;
        try { actual=((Integer)method.invoke(null,arguments)).intValue(); }
        catch(InvocationTargetException e) {
            Throwable cause=e.getCause(); if(cause instanceof Exception)throw(Exception)cause;
            if(cause instanceof Error)throw(Error)cause; throw new AssertionError(cause);
        }
        System.out.println("CASE method="+name+" skip="+skip+" value="+value+" counter="+counter+
            " actual="+actual+" expected="+expected);
        if(actual!=expected)throw new AssertionError(name+" differs from finite source oracle");
    }
}
