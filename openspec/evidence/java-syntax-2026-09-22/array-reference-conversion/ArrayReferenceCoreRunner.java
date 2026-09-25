public class ArrayReferenceCoreRunner {
    private static String error(Throwable error) {
        return error.getClass().getName();
    }

    private static void line(String name, String value) {
        System.out.println(name + "=" + value);
    }

    private static void call(String name, Call call) {
        try {
            line(name, call.run());
        } catch (Throwable error) {
            line(name, "error:" + error(error));
        }
    }

    private interface Call {
        String run();
    }

    public static void main(String[] args) {
        final int[][] intMatrix = new int[2][];
        intMatrix[0] = new int[0];
        intMatrix[1] = new int[3];
        final String[] strings = new String[2];
        final String[][] stringMatrix = new String[2][];
        stringMatrix[0] = new String[0];
        stringMatrix[1] = new String[4];

        call("target:int[][]", new Call() {
            public String run() {
                return ArrayReferenceCore.typedIntMatrix(intMatrix);
            }
        });
        call("target:String[]", new Call() {
            public String run() {
                return ArrayReferenceCore.typedStringArray(strings);
            }
        });
        call("target:String[][]", new Call() {
            public String run() {
                return ArrayReferenceCore.typedStringMatrix(stringMatrix);
            }
        });
        call("target:Object-widened-int[][]", new Call() {
            public String run() {
                Object value = intMatrix;
                return ArrayReferenceCore.widenedObject(value);
            }
        });
        call("target:Object[]-widened-int[][]", new Call() {
            public String run() {
                Object[] value = intMatrix;
                return ArrayReferenceCore.widenedObjectArray(value);
            }
        });
        call("target:Object-widened-String[]", new Call() {
            public String run() {
                Object value = strings;
                return ArrayReferenceCore.widenedObject(value);
            }
        });
        call("target:Object[]-widened-String[]", new Call() {
            public String run() {
                Object[] value = strings;
                return ArrayReferenceCore.widenedObjectArray(value);
            }
        });
        call("target:Object-widened-String[][]", new Call() {
            public String run() {
                Object value = stringMatrix;
                return ArrayReferenceCore.widenedObject(value);
            }
        });
        call("target:Object[]-widened-String[][]", new Call() {
            public String run() {
                Object[] value = stringMatrix;
                return ArrayReferenceCore.widenedObjectArray(value);
            }
        });

        call("exact:int[][]", new Call() {
            public String run() {
                return ArrayReferenceCore.exactIntMatrix(intMatrix);
            }
        });
        call("exact:String[]", new Call() {
            public String run() {
                return ArrayReferenceCore.exactStringArray(strings);
            }
        });
        call("exact:String[][]", new Call() {
            public String run() {
                return ArrayReferenceCore.exactStringMatrix(stringMatrix);
            }
        });
        call("exact:Object[]-int[][]", new Call() {
            public String run() {
                return ArrayReferenceCore.exactObjectArray(intMatrix);
            }
        });
        call("exact:Object[]-String[]", new Call() {
            public String run() {
                return ArrayReferenceCore.exactObjectArray(strings);
            }
        });
        call("exact:Object[]-String[][]", new Call() {
            public String run() {
                return ArrayReferenceCore.exactObjectArray(stringMatrix);
            }
        });
        call("exact:Object[][]-String[][]", new Call() {
            public String run() {
                return ArrayReferenceCore.exactObjectMatrix(stringMatrix);
            }
        });
        call("describe:Object[]-int[][]", new Call() {
            public String run() {
                return ArrayReferenceCore.describeObjectArray(intMatrix);
            }
        });
        call("describe:Object[]-String[]", new Call() {
            public String run() {
                return ArrayReferenceCore.describeObjectArray(strings);
            }
        });
        call("describe:Object[][]-String[][]", new Call() {
            public String run() {
                return ArrayReferenceCore.describeObjectMatrix(stringMatrix);
            }
        });

        final int[][] emptyInt = new int[0][];
        final String[] emptyStrings = new String[0];
        final String[][] emptyStringMatrix = new String[0][];
        call("size:empty-int[][]", new Call() {
            public String run() {
                return ArrayReferenceCore.exactIntMatrix(emptyInt);
            }
        });
        call("size:empty-String[]", new Call() {
            public String run() {
                return ArrayReferenceCore.exactStringArray(emptyStrings);
            }
        });
        call("size:empty-String[][]", new Call() {
            public String run() {
                return ArrayReferenceCore.exactStringMatrix(emptyStringMatrix);
            }
        });

        final int[][] intNull = null;
        final String[] stringNull = null;
        final String[][] stringMatrixNull = null;
        call("null:exact-int[][]", new Call() {
            public String run() {
                return ArrayReferenceCore.exactIntMatrix(intNull);
            }
        });
        call("null:exact-String[]", new Call() {
            public String run() {
                return ArrayReferenceCore.exactStringArray(stringNull);
            }
        });
        call("null:exact-String[][]", new Call() {
            public String run() {
                return ArrayReferenceCore.exactStringMatrix(stringMatrixNull);
            }
        });
        call("null:Object-widened", new Call() {
            public String run() {
                Object value = intNull;
                return ArrayReferenceCore.widenedObject(value);
            }
        });
        call("null:Object[]-widened", new Call() {
            public String run() {
                Object[] value = stringMatrixNull;
                return ArrayReferenceCore.widenedObjectArray(value);
            }
        });
        call("null:Object[]-dereference", new Call() {
            public String run() {
                return ArrayReferenceCore.describeObjectArray(stringNull);
            }
        });
    }
}
