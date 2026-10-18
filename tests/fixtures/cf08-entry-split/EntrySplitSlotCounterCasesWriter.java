import java.io.ByteArrayOutputStream;
import java.io.DataOutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/** Writes one bounded classfile control; execution/verification is performed separately. */
public final class EntrySplitSlotCounterCasesWriter {
    private static final class MethodSpec {
        final String name, descriptor;
        final int maxStack, maxLocals;
        final byte[] code, frames;
        MethodSpec(String n, String d, int s, int l, byte[] c, byte[] f) {
            name=n; descriptor=d; maxStack=s; maxLocals=l; code=c; frames=f;
        }
    }
    private static final class Pool {
        final List<byte[]> entries = new ArrayList<>();
        final Map<String,Integer> ids = new HashMap<>();
        int add(String key, byte[] bytes) {
            Integer old=ids.get(key); if(old!=null)return old;
            entries.add(bytes); int id=entries.size(); ids.put(key,id); return id;
        }
        int utf(String s) throws Exception {
            ByteArrayOutputStream b=new ByteArrayOutputStream(); DataOutputStream o=new DataOutputStream(b);
            o.writeByte(1); o.writeUTF(s); return add("U"+s,b.toByteArray());
        }
        int clazz(String s) throws Exception {
            int u=utf(s); return add("C"+s,new byte[]{7,(byte)(u>>>8),(byte)u});
        }
        int nat(String n,String d) throws Exception {
            int a=utf(n),b=utf(d); return add("N"+n+"\0"+d,new byte[]{12,(byte)(a>>>8),(byte)a,(byte)(b>>>8),(byte)b});
        }
        int method(String c,String n,String d) throws Exception {
            int a=clazz(c),b=nat(n,d); return add("M"+c+"."+n+d,new byte[]{10,(byte)(a>>>8),(byte)a,(byte)(b>>>8),(byte)b});
        }
        void write(DataOutputStream o) throws Exception {
            o.writeShort(entries.size()+1); for(byte[] e:entries)o.write(e);
        }
    }

    public static void main(String[] args) throws Exception {
        if(args.length!=1) throw new IllegalArgumentException("usage: EntrySplitSlotCounterCasesWriter <class-root>");
        Path root=Paths.get(args[0]);
        Pool cp=new Pool();
        int thisClass=cp.clazz("cf08/EntrySplitSlotCounterCases");
        int superClass=cp.clazz("java/lang/Object");
        int initName=cp.utf("<init>"), initDesc=cp.utf("()V"), initRef=cp.method("java/lang/Object","<init>","()V");
        int codeName=cp.utf("Code"), stackName=cp.utf("StackMapTable");
        int sourceName=cp.utf("SourceFile"), sourceFile=cp.utf("EntrySplitSlotCounterCasesWriter.java");
        List<MethodSpec> methods=new ArrayList<>();
        // skip@0, P=iinc slot2@4, H=slot1 test@7, L=iinc slot1@11, X=slot2@17.
        methods.add(new MethodSpec("permuted","(ZII)I",1,3,
            new byte[]{0x1a,(byte)0x9a,0,6,(byte)0x84,2,7,0x1b,(byte)0x99,0,9,
                (byte)0x84,1,(byte)0xff,(byte)0xa7,(byte)0xff,(byte)0xf6,0x1c,(byte)0xac},
            new byte[]{0,3,4,2,9}));
        // value slot0, skip slot1, counter slot2; loop while counter is nonzero and increment to zero.
        methods.add(new MethodSpec("countUp","(IZI)I",1,3,
            new byte[]{0x1b,(byte)0x9a,0,6,(byte)0x84,0,3,0x1c,(byte)0x99,0,9,
                (byte)0x84,2,1,(byte)0xa7,(byte)0xff,(byte)0xf6,0x1a,(byte)0xac},
            new byte[]{0,3,4,2,9}));
        for(MethodSpec m:methods) { cp.utf(m.name); cp.utf(m.descriptor); }

        ByteArrayOutputStream bytes=new ByteArrayOutputStream(); DataOutputStream out=new DataOutputStream(bytes);
        out.writeInt(0xcafebabe); out.writeShort(0); out.writeShort(52); cp.write(out);
        out.writeShort(0x0031); out.writeShort(thisClass); out.writeShort(superClass);
        out.writeShort(0); out.writeShort(0); out.writeShort(methods.size()+1);
        writeMethod(out,initName,initDesc,0x0001,1,1,
            new byte[]{0x2a,(byte)0xb7,(byte)(initRef>>>8),(byte)initRef,(byte)0xb1},null,codeName,stackName);
        for(MethodSpec m:methods)
            writeMethod(out,cp.utf(m.name),cp.utf(m.descriptor),0x0009,m.maxStack,m.maxLocals,m.code,m.frames,codeName,stackName);
        out.writeShort(1); out.writeShort(sourceName); out.writeInt(2); out.writeShort(sourceFile); out.flush();
        Path target=root.resolve("cf08/EntrySplitSlotCounterCases.class");
        Files.createDirectories(target.getParent()); Files.write(target,bytes.toByteArray());
    }

    private static void writeMethod(DataOutputStream out,int name,int desc,int access,int maxStack,int maxLocals,
            byte[] code,byte[] frames,int codeName,int stackName) throws Exception {
        ByteArrayOutputStream b=new ByteArrayOutputStream(); DataOutputStream c=new DataOutputStream(b);
        c.writeShort(maxStack); c.writeShort(maxLocals); c.writeInt(code.length); c.write(code); c.writeShort(0);
        c.writeShort(frames==null?0:1);
        if(frames!=null) { c.writeShort(stackName); c.writeInt(frames.length); c.write(frames); }
        c.flush(); out.writeShort(access); out.writeShort(name); out.writeShort(desc);
        out.writeShort(1); out.writeShort(codeName); out.writeInt(b.size()); out.write(b.toByteArray());
    }
}
