import java.lang.reflect.*;
import java.util.*;
public class BoundaryReflection {
  public static void main(String[] args) throws Exception {
    for (String name : new String[] {"identity", "bodyPositive", "bodyOverload"}) {
      Method m = Boundaries.class.getMethod(name, List.class);
      System.out.println(name + " parameter=" + m.getGenericParameterTypes()[0].getTypeName()
          + " return=" + m.getGenericReturnType().getTypeName());
    }
    Method inner = AmbiguousInnerBoundary.class.getMethod("left", List.class);
    System.out.println("left parameter=" + inner.getGenericParameterTypes()[0].getTypeName()
        + " return=" + inner.getGenericReturnType().getTypeName());
    Method annotated = AnnotationPathBoundary.class.getMethod("identity", List.class);
    AnnotatedParameterizedType type = (AnnotatedParameterizedType) annotated.getAnnotatedParameterTypes()[0];
    System.out.println("annotated parameter=" + annotated.getGenericParameterTypes()[0].getTypeName()
        + " type-argument-annotations=" + Arrays.toString(type.getAnnotatedActualTypeArguments()[0].getAnnotations()));
    Method classVariable = ClassVariableBoundary.class.getMethod("identity", Object.class);
    System.out.println("class-variable parameter=" + classVariable.getGenericParameterTypes()[0].getTypeName()
        + " return=" + classVariable.getGenericReturnType().getTypeName());
  }
}
