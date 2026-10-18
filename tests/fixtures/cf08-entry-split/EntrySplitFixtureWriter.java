import java.io.ByteArrayOutputStream;
import java.io.DataOutputStream;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/** Emits candidate classfile controls for later verifier inspection; no target source is synthesized. */
public final class EntrySplitFixtureWriter {
    private static final class MethodSpec {
        final String name, descriptor;
        final int access, maxStack, maxLocals;
        final byte[] code, frames;
        int handlerStart=-1, handlerEnd=-1, handlerBci=-1, catchType;
        MethodSpec(String n, String d, int a, int s, int l, byte[] c, byte[] f) {
            name=n; descriptor=d; access=a; maxStack=s; maxLocals=l; code=c; frames=f;
        }
    }
    private static final class Pool {
        final List<byte[]> entries = new ArrayList<>();
        final Map<String,Integer> ids = new HashMap<>();
        int add(String key, byte[] bytes) { Integer old=ids.get(key); if(old!=null)return old; entries.add(bytes); int id=entries.size(); ids.put(key,id); return id; }
        int utf(String s) throws IOException { ByteArrayOutputStream b=new ByteArrayOutputStream(); DataOutputStream o=new DataOutputStream(b); o.writeByte(1); o.writeUTF(s); return add("U"+s,b.toByteArray()); }
        int clazz(String s) throws IOException { int u=utf(s); return add("C"+s,new byte[]{7,(byte)(u>>>8),(byte)u}); }
        int nat(String n,String d) throws IOException { int a=utf(n),b=utf(d); return add("N"+n+"\0"+d,new byte[]{12,(byte)(a>>>8),(byte)a,(byte)(b>>>8),(byte)b}); }
        int method(String c,String n,String d) throws IOException { int a=clazz(c),b=nat(n,d); return add("M"+c+"."+n+d,new byte[]{10,(byte)(a>>>8),(byte)a,(byte)(b>>>8),(byte)b}); }
        void write(DataOutputStream o) throws IOException { o.writeShort(entries.size()+1); for(byte[] e:entries)o.write(e); }
    }
    private static final int PUBLIC_STATIC=0x0009;
    private static final int[] SAME_FRAMES={4,2,9};

    public static void main(String[] args) throws Exception {
        if(args.length!=1) throw new IllegalArgumentException("usage: EntrySplitFixtureWriter <class-root>");
        Path root=Paths.get(args[0]);
        write(root,"cf08/EntrySplitCases.class","cf08/EntrySplitCases",positive());
        write(root,"cf08/EntrySplitRefusals.class","cf08/EntrySplitRefusals",refusals());
        write(root,"cf08/EntrySplitTopologyRefusals.class","cf08/EntrySplitTopologyRefusals",topologyRefusals());
        write(root,"cf08/EntrySplitExceptionRefusal.class","cf08/EntrySplitExceptionRefusal",exceptionRefusal());
        writeLegacy(root);
    }
    private static List<MethodSpec> positive() {
        List<MethodSpec> m=new ArrayList<>();
        m.add(new MethodSpec("test","(IZI)I",PUBLIC_STATIC,1,3,
            new byte[]{0x1b,(byte)0x9a,0,6,(byte)0x84,0,3,0x1c,(byte)0x99,0,9,(byte)0x84,2,(byte)0xff,(byte)0xa7,(byte)0xff,(byte)0xf6,0x1a,(byte)0xac},
            new byte[]{0,3,4,2,9}));
        m.add(new MethodSpec("signed","(IZI)I",PUBLIC_STATIC,1,3,
            new byte[]{0x1b,(byte)0x9a,0,6,(byte)0x84,0,(byte)0xfb,0x1c,(byte)0x99,0,9,(byte)0x84,2,(byte)0xff,(byte)0xa7,(byte)0xff,(byte)0xf6,0x1a,(byte)0xac},
            new byte[]{0,3,4,2,9}));
        m.add(new MethodSpec("shifted","(IIIZI)I",PUBLIC_STATIC,1,5,
            new byte[]{0x1d,(byte)0x9a,0,6,(byte)0x84,0,5,0x15,4,(byte)0x99,0,9,(byte)0x84,4,(byte)0xff,(byte)0xa7,(byte)0xff,(byte)0xf5,0x1a,(byte)0xac},
            new byte[]{0,3,4,2,10}));
        m.add(new MethodSpec("inverted","(IZI)I",PUBLIC_STATIC,1,3,
            new byte[]{0x1b,(byte)0x99,0,13,0x1c,(byte)0x99,0,15,(byte)0x84,2,(byte)0xff,(byte)0xa7,0,3,(byte)0x84,0,(byte)0xfe,(byte)0xa7,(byte)0xff,(byte)0xf3,0x1a,(byte)0xac},
            new byte[]{0,3,4,9,5}));
        return m;
    }
    private static List<MethodSpec> refusals() {
        List<MethodSpec> m=new ArrayList<>();
        m.add(new MethodSpec("booleanPrefix","(ZZI)I",PUBLIC_STATIC,1,3,
            new byte[]{0x1b,(byte)0x9a,0,6,(byte)0x84,0,1,0x1c,(byte)0x99,0,9,(byte)0x84,2,(byte)0xff,(byte)0xa7,(byte)0xff,(byte)0xf6,0x1a,(byte)0xac},new byte[]{0,3,4,2,9}));
        m.add(new MethodSpec("bytePrefix","(BZI)I",PUBLIC_STATIC,1,3,
            new byte[]{0x1b,(byte)0x9a,0,6,(byte)0x84,0,5,0x1c,(byte)0x99,0,9,(byte)0x84,2,(byte)0xff,(byte)0xa7,(byte)0xff,(byte)0xf6,0x1a,(byte)0xac},new byte[]{0,3,4,2,9}));
        m.add(new MethodSpec("alias","(IZ)I",PUBLIC_STATIC,1,2,
            new byte[]{0x1b,(byte)0x9a,0,6,(byte)0x84,0,1,0x1a,(byte)0x99,0,9,(byte)0x84,0,(byte)0xff,(byte)0xa7,(byte)0xff,(byte)0xf6,0x1a,(byte)0xac},new byte[]{0,3,4,2,9}));
        // Math.incrementExact is a valid call with observable overflow behavior; the first slice rejects calls in P.
        m.add(new MethodSpec("callPrefix","(IZI)I",PUBLIC_STATIC,1,3,
            new byte[]{0x1b,(byte)0x9a,0,8,0x1a,(byte)0xb8,0,0,(byte)0x3b,0x1c,(byte)0x99,0,9,(byte)0x84,2,(byte)0xff,(byte)0xa7,(byte)0xff,(byte)0xf4,0x1a,(byte)0xac},new byte[]{0,3,4,4,9}));
        return m;
    }
    private static List<MethodSpec> topologyRefusals() {
        List<MethodSpec> m=new ArrayList<>();
        // Code is 26 B: preheader branches/goto at 1->11, 5->14, 8->18; SCC P=11, H=14, L=18, X=24. Frames: 11,14,18,24.
        // Three preheader successors enter the SCC at P(11), H(14), and L(18).
        m.add(new MethodSpec("extraEntry","(IZI)I",PUBLIC_STATIC,1,3,
            new byte[]{0x1b,(byte)0x99,0,10,0x1c,(byte)0x99,0,9,(byte)0xa7,0,10,(byte)0x84,0,3,0x1c,(byte)0x99,0,9,(byte)0x84,2,(byte)0xff,(byte)0xa7,(byte)0xff,(byte)0xf6,0x1a,(byte)0xac},
            new byte[]{0,4,11,2,3,5}));
        // Code is 25 B: ifne@1->7, ifeq@8->23, ifeq@15->21, goto@18->4; returns at 21 and 23. Frames: 4,7,21,23.
        // The loop has two reachable exits and two return blocks.
        m.add(new MethodSpec("extraExit","(IZI)I",PUBLIC_STATIC,1,3,
            new byte[]{0x1b,(byte)0x9a,0,6,(byte)0x84,0,3,0x1c,(byte)0x99,0,15,(byte)0x84,2,(byte)0xff,0x1c,(byte)0x99,0,6,(byte)0xa7,(byte)0xff,(byte)0xf2,0x1a,(byte)0xac,0x1a,(byte)0xac},
            new byte[]{0,4,4,2,13,1}));
        // Code is 29 B: prefix@4, header@7, counter latch@11->4, exit dispatcher@17, live side node@21, return@27. Frames: 4,7,17,27.
        // A live side node lies after the loop exit and before the returned value.
        m.add(new MethodSpec("extraLive","(IZZI)I",PUBLIC_STATIC,1,4,
            new byte[]{0x1b,(byte)0x9a,0,6,(byte)0x84,0,3,0x1d,(byte)0x99,0,9,(byte)0x84,3,(byte)0xff,(byte)0xa7,(byte)0xff,(byte)0xf6,0x1c,(byte)0x9a,0,9,(byte)0x84,0,1,(byte)0xa7,0,3,0x1a,(byte)0xac},
            new byte[]{0,4,4,2,9,9}));
        // Code is 18 B: the value remains on stack across ifne@2->7, test@8->17, latch@14->5; stack-int frames at 5,7,17.
        // Value remains on the operand stack from the entry test through exit.
        m.add(new MethodSpec("stackEscape","(IZI)I",PUBLIC_STATIC,2,3,
            new byte[]{0x1a,0x1b,(byte)0x9a,0,5,0x06,0x60,0x1c,(byte)0x99,0,9,(byte)0x84,2,(byte)0xff,(byte)0xa7,(byte)0xff,(byte)0xf7,(byte)0xac},
            new byte[]{0,3,69,1,65,1,73,1}));
        return m;
    }
    private static List<MethodSpec> exceptionRefusal() {
        List<MethodSpec> m=new ArrayList<>();
        // Prefix performs idiv in [4,8); the exceptional edge reaches a separate return handler.
        MethodSpec exceptional=new MethodSpec("exceptionPrefix","(IZII)I",PUBLIC_STATIC,2,4,
            new byte[]{0x1b,(byte)0x9a,0,7,0x1a,0x1d,0x6c,0x3b,0x1c,(byte)0x99,0,9,(byte)0x84,2,(byte)0xff,(byte)0xa7,(byte)0xff,(byte)0xf5,0x1a,(byte)0xac,0x57,0x10,99,(byte)0xac},
            new byte[]{0,4,4,3,9,(byte)0xff,0,1,0,4,1,1,1,1,0,1,7,0,0});
        exceptional.handlerStart=4; exceptional.handlerEnd=8; exceptional.handlerBci=20;
        m.add(exceptional);
        return m;
    }
    private static void writeLegacy(Path root) throws Exception {
        List<MethodSpec> m=new ArrayList<>();
        // Code is 31 B: ifne@1->10, jsr@4->25, ifeq@11->23, jsr@17->25, ret@29; classfile major 49 only.
        // Version 49 legacy subroutine control. This does not claim Java 8 support.
        m.add(new MethodSpec("legacy","(IZI)I",PUBLIC_STATIC,1,4,
            new byte[]{0x1b,(byte)0x9a,0,9,(byte)0xa8,0,21,(byte)0xa7,0,3,0x1c,(byte)0x99,0,12,(byte)0x84,2,(byte)0xff,(byte)0xa8,0,8,(byte)0xa7,(byte)0xff,(byte)0xf6,0x1a,(byte)0xac,0x4e,(byte)0x84,0,3,(byte)0xa9,3},null));
        writeVersion(root,"cf08/EntrySplitLegacyCases.class","cf08/EntrySplitLegacyCases",m,49);
    }
    private static void write(Path root,String path,String internal,List<MethodSpec> methods) throws Exception { writeVersion(root,path,internal,methods,52); }
    private static void writeVersion(Path root,String path,String internal,List<MethodSpec> methods,int major) throws Exception {
        Pool cp=new Pool();
        int thisClass=cp.clazz(internal), superClass=cp.clazz("java/lang/Object");
        int initName=cp.utf("<init>"), initDesc=cp.utf("()V"), initRef=cp.method("java/lang/Object","<init>","()V");
        int codeName=cp.utf("Code"), stackName=cp.utf("StackMapTable"), sourceName=cp.utf("SourceFile"), sourceFile=cp.utf("EntrySplitFixtureWriter.java");
        for(MethodSpec m:methods){cp.utf(m.name);cp.utf(m.descriptor);}
        int mathRef=cp.method("java/lang/Math","incrementExact","(I)I");
        int throwableClass=cp.clazz("java/lang/ArithmeticException");
        // Patch the exception frame's Object verification type after pool identities are fixed.
        for(MethodSpec m:methods)if(m.name.equals("exceptionPrefix")){
            m.catchType=throwableClass;
            m.frames[m.frames.length-2]=(byte)(throwableClass>>>8);m.frames[m.frames.length-1]=(byte)throwableClass;
        }
        // Patch the deliberately unresolved call operands with the constant-pool index.
        for(MethodSpec m:methods) if(m.name.equals("callPrefix")){m.code[6]=(byte)(mathRef>>>8);m.code[7]=(byte)mathRef;}
        ByteArrayOutputStream bytes=new ByteArrayOutputStream(); DataOutputStream out=new DataOutputStream(bytes);
        out.writeInt(0xcafebabe);out.writeShort(0);out.writeShort(major);cp.write(out);
        out.writeShort(0x0031);out.writeShort(thisClass);out.writeShort(superClass);out.writeShort(0);out.writeShort(0);
        out.writeShort(methods.size()+1);
        writeMethod(out,initName,initDesc,0x0001,1,1,new byte[]{0x2a,(byte)0xb7,(byte)(initRef>>>8),(byte)initRef,(byte)0xb1},null,-1,-1,-1,0,codeName,stackName);
        for(MethodSpec m:methods)writeMethod(out,cp.utf(m.name),cp.utf(m.descriptor),m.access,m.maxStack,m.maxLocals,m.code,m.frames,m.handlerStart,m.handlerEnd,m.handlerBci,m.catchType,codeName,stackName);
        out.writeShort(1);out.writeShort(sourceName);out.writeInt(2);out.writeShort(sourceFile);out.flush();
        Path target=root.resolve(path);Files.createDirectories(target.getParent());Files.write(target,bytes.toByteArray());
    }
    private static void writeMethod(DataOutputStream out,int name,int desc,int access,int maxStack,int maxLocals,byte[] code,byte[] frames,int handlerStart,int handlerEnd,int handlerBci,int catchType,int codeName,int stackName)throws Exception{
        ByteArrayOutputStream b=new ByteArrayOutputStream();DataOutputStream c=new DataOutputStream(b);c.writeShort(maxStack);c.writeShort(maxLocals);c.writeInt(code.length);c.write(code);c.writeShort(handlerStart<0?0:1);if(handlerStart>=0){c.writeShort(handlerStart);c.writeShort(handlerEnd);c.writeShort(handlerBci);c.writeShort(catchType);}c.writeShort(frames==null?0:1);
        if(frames!=null){c.writeShort(stackName);c.writeInt(frames.length);c.write(frames);}c.flush();out.writeShort(access);out.writeShort(name);out.writeShort(desc);out.writeShort(1);out.writeShort(codeName);out.writeInt(b.size());out.write(b.toByteArray());
    }
}
