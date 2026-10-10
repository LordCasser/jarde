package defpackage;
public class Runner { public static void main(String[] args) { long sum=0; for(int i=0;i<=65535;i++) { int value=Meet.viaStore((char)i); if(value!=i) throw new AssertionError("char="+i+", result="+value); sum+=value; } System.out.println("chars=65536,sum="+sum); } }
