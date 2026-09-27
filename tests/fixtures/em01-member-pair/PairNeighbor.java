import java.nio.file.Files;
import java.nio.file.Paths;
import jdk.internal.org.objectweb.asm.ClassReader;
import jdk.internal.org.objectweb.asm.ClassWriter;
import jdk.internal.org.objectweb.asm.Opcodes;
import jdk.internal.org.objectweb.asm.Type;
import jdk.internal.org.objectweb.asm.tree.*;

public class PairNeighbor {
    public static void main(String[] args) throws Exception {
        if (args.length != 3) throw new IllegalArgumentException("MODE INPUT OUTPUT");
        ClassNode node = new ClassNode();
        new ClassReader(Files.readAllBytes(Paths.get(args[1]))).accept(node, 0);
        switch (args[0]) {
            case "wrong-self": {
                boolean found = false;
                for (InnerClassNode row : node.innerClasses) {
                    if (row.name.equals("em01/Shape$I")) {
                        row.outerName = "em01/Other";
                        found = true;
                    }
                }
                if (!found) throw new IllegalStateException("missing self row");
                break;
            }
            case "third-child":
                node.innerClasses.add(new InnerClassNode(
                        "em01/Shape$Extra", "em01/Shape", "Extra", Opcodes.ACC_PUBLIC | Opcodes.ACC_STATIC));
                break;
            case "interface-default":
            case "interface-static": {
                MethodNode method = method(node, "test3");
                method.access = Opcodes.ACC_PUBLIC | (args[0].equals("interface-static") ? Opcodes.ACC_STATIC : 0);
                method.instructions.add(new InsnNode(Opcodes.ICONST_0));
                method.instructions.add(new InsnNode(Opcodes.IRETURN));
                break;
            }
            case "field":
                node.fields.add(new FieldNode(Opcodes.ACC_PUBLIC | Opcodes.ACC_STATIC | Opcodes.ACC_FINAL,
                        "extra", "I", null, 1));
                break;
            case "signature":
                node.signature = "Ljava/lang/Object;";
                break;
            case "root-use": {
                MethodNode constructor = method(node, "<init>");
                AbstractInsnNode call = constructor.instructions.getFirst();
                while (call != null && call.getOpcode() != Opcodes.INVOKESPECIAL) call = call.getNext();
                if (call == null) throw new IllegalStateException("missing super call");
                InsnList use = new InsnList();
                use.add(new LdcInsnNode(Type.getObjectType("em01/Shape$I")));
                use.add(new InsnNode(Opcodes.POP));
                constructor.instructions.insert(call, use);
                break;
            }
            default: throw new IllegalArgumentException(args[0]);
        }
        ClassWriter writer = new ClassWriter(ClassWriter.COMPUTE_FRAMES | ClassWriter.COMPUTE_MAXS);
        node.accept(writer);
        Files.write(Paths.get(args[2]), writer.toByteArray());
    }

    private static MethodNode method(ClassNode node, String name) {
        for (MethodNode method : node.methods) {
            if (method.name.equals(name)) return method;
        }
        throw new IllegalStateException("missing " + name);
    }
}
