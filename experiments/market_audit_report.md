# J2ME Market Batch Compatibility & Stress Test Audit Report
**Date:** 2026-10-30 01:22:28 | **Total Games Tested:** 500/7328 | **Elapsed:** 71.34s

## 1. High-Level Category Distribution
| Category | Count | Percentage |
|:---|:---:|:---:|
| **COMPLETED_WINDOW** | 408 | 81.6% |
| **TIMEOUT_DEADLOCK** | 74 | 14.8% |
| **GUEST_EXCEPTION** | 17 | 3.4% |
| **PARSER_ERROR** | 1 | 0.2% |

## 2. Top Uncaught Guest Exceptions
| Exception Class | Count | % of Exceptions |
|:---|:---:|:---:|
| `java/lang/NullPointerException` | 14 | 82.4% |
| `java/lang/RuntimeException` | 2 | 11.8% |
| `java/lang/ArrayIndexOutOfBoundsException` | 1 | 5.9% |

## 3. Top Detailed Exception Signatures
| Occurrences | Signature & Location |
|:---:|:---|
| **1x** | `java/lang/RuntimeException:  at ax.a()Lax;:7` |
| **1x** | `java/lang/RuntimeException:  at v.a(Ljava/lang/String;S)Ljava/lang/Object;:29` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during InvokeVirtual { class_name: "javax/microedition/lcdui/Display", method_name: "setCurrent", descriptor: "(Ljavax/microedition/lcdui/Displayable;)V" } at c.a(Ljavax/microedition/lcdui/Displayable;)V (pc=4) at c.a(Ljavax/microedition/lcdui/Displayable;)V:4` |
| **1x** | `java/lang/ArrayIndexOutOfBoundsException: Array index out of bounds during AAStore at c.a()V (pc=141) at c.a()V:141` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during IALoad at HG.d()V (pc=38) at HG.d()V:38` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during InvokeVirtual { class_name: "javax/microedition/lcdui/Image", method_name: "getWidth", descriptor: "()I" } at AntiTerroristUnit.startApp()V (pc=203) at AntiTerroristUnit.startApp()V:203` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during InvokeVirtual { class_name: "javax/microedition/lcdui/Image", method_name: "getWidth", descriptor: "()I" } at AntiTerroristUnit.initialize()V (pc=9) at AntiTerroristUnit.initialize()V:9` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during IALoad at C.paint(Ljavax/microedition/lcdui/Graphics;)V (pc=10) at C.paint(Ljavax/microedition/lcdui/Graphics;)V:10` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during IALoad at C.paint(Ljavax/microedition/lcdui/Graphics;)V (pc=22) at C.paint(Ljavax/microedition/lcdui/Graphics;)V:22` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during IALoad at i.d(Ljavax/microedition/lcdui/Graphics;)V (pc=3) at i.d(Ljavax/microedition/lcdui/Graphics;)V:3` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during InvokeVirtual { class_name: "java/lang/String", method_name: "equals", descriptor: "(Ljava/lang/Object;)Z" } at a/a/g.a(Ljavax/microedition/lcdui/Graphics;I)V (pc=31) at a/a/g.a(Ljavax/microedition/lcdui/Graphics;I)V:31` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during GetField { class_name: "n", field_name: "d", descriptor: "B" } at com/playw/alexander/AlexanderMIDlet.a(Ljava/lang/String;B)Li; (pc=304) at com/playw/alexander/AlexanderMIDlet.a(Ljava/lang/String;B)Li;:304` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during AALoad at f.a(IIIII)V (pc=18) at f.a(IIIII)V:18` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during InvokeVirtual { class_name: "javax/microedition/lcdui/Image", method_name: "getWidth", descriptor: "()I" } at com/zenops/aaf/g.paint(Ljavax/microedition/lcdui/Graphics;)V (pc=18) at com/zenops/aaf/g.paint(Ljavax/microedition/lcdui/Graphics;)V:18` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during SALoad at a.b(Ljava/lang/String;)I (pc=23) at a.b(Ljava/lang/String;)I:23` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during IALoad at G.paint(Ljavax/microedition/lcdui/Graphics;)V (pc=121) at G.paint(Ljavax/microedition/lcdui/Graphics;)V:121` |
| **1x** | `java/lang/NullPointerException: Null pointer dereference during InvokeVirtual { class_name: "java/lang/String", method_name: "charAt", descriptor: "(I)C" } at aquaria/a.b(Ljava/lang/String;)V (pc=409) at aquaria/a.b(Ljava/lang/String;)V:409` |

## 4. Top Parser & Loader Errors
| Occurrences | Error Description |
|:---:|:---|
| **1x** | `Emulator exited with code -6 without output JSON` |
