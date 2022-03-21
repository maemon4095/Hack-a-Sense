---
title: C#で高級な列挙型を作る．
tags: csharp
---

{% include table_of_contents.md %}

```plaintext
enum Option<T>
{
    None, Some(T value)
}
```

こんな事したいよねって話．でもC#の列挙型はこんなことできない，からどうにかしたい．

一応デフォルトの列挙型でも値を紐づけることは可能で，属性と拡張メソッドを使って下記のようにします．

```cs
enum EnumWithValue
{
    [Value("value")]
    Key
}

static class EnumWithValueExtension
{
    public static string Value()
    {
        return "value";
    }
}
//呼び出し側
Console.WriteLine(EnumWithValue.Key.Value());// value

```

でもこれで紐づけられるのは基本的にコンパイル時に定数になる値だけになるし，ジェネリクスにもできません．

次に候補になるのが構造体で実装を与える案．これは結構一般的な手法で，enumライクに扱える構造体を作ることはあったりなかったり．作るとこんな感じです．

```cs
readonly struct EnumLike
{
    EnumLike(int num)
    {
        this.num = num;
    }

    //普通のenumと同じ感じ
    public static EnumLike Default => new EnumLike(0);

    //8bit分でフラグ，残りで値を持つとか
    public static EnumLike Argument(int mode)
    {
        return new EnumLike((mode << 8) | 1);
    }

    readonly int num;
}

//呼び出し側
Console.WriteLine(EnumLike.Argument(2));
```

この構造体での実装はなかなか優秀で，enumの軽量さを保ったまま追加の機能を追加できてジェネリクスも可能．でもIEquatbleとか演算子等々，結構手間がかかるのが玉に瑕です．

というわけで値を持つ列挙型は，型の自動生成で作成します．C#には構造体とクラスの両方があって，enumに似せることを考えると構造体ベースが使いやすいです．でも構造体ベースだけだとOCamlみたいに`type 'a tree = Leaf of 'a | Node of 'a * 'a tree * 'a tree`こんな感じで再帰的な構造を作れなかったり，クラスベースだと実装が非常に簡単になります．

# 実装

SourceGeneratorでマーカーとして`EnumerationAttribute`がついた部分型から自動生成します．
ガワの仕様を具体例として`void`, `int`か`string`を持つ`Enum`型から考えます．
```cs
//元コード
[Enumeration]
partial struct Enum
{
    public static partial Enum Key0(int num);
    public static partial Enum Key1(string str);
}

//生成
Enum key0 = Enum.Key0(int num);
Enum key1 = Enum.Key1(string str);

//分解
key0.Key0(out int num);   //== true
key0.Key1(out string str);//== false

key1.Key0(out int num);   //== false
key1.Key1(out string str);//== true

key0.Type//== Enum.Case.Key0
```
使い心地としてはこんな感じで，生成するメソッドから対になる分解のメソッドを自動生成します．`switch`で分岐できるように`enum`も生成します

ここでポイントがあって，C#では同じ名前，引数を持つ関数をスタティック，インスタンスで呼び分けることが出来ません．

なので，値を持たないキーに対しては素朴に関数を作ってしまうと，コンパイルエラーになります．
```cs
class A
{
    public static A Void();
    public bool Void();//コンパイルエラー
}
```
この問題は分解のメソッドも静的にすることで解決します．
```cs
class A
{
    public static A Void();
    public static bool Void(A __self);
}
//呼び出し側
var instance = A.Void();
A.Void(instance);//== true
```
ですが，このままでは分解が手間なので拡張メソッドを用います．
```cs
class A
{
    public static A Void();
    public static bool Void(A __self);
}

static class A_Extension
{
    public static bool Void(this A __self) => A.Void(__self);
}

//呼び出し側
var instance = A.Void();
instance.Void();//== true
```
~~こうやって呼び分けできるなら初めからさせてくれい~~

というわけで生成と分解をどちらも静的に実装し，拡張メソッドとして分解を呼ぶ様にします．

ガワの仕様が決まったので，次は内部について考えます．


## 具体例

内部実装は本質的には共用体と同等で，クラスと構造体で大幅に異なった実装になります．それぞれについて型の生成結果から考えていきます．

### クラスベース実装
クラスベース実装では抽象クラスを用います．`Encoding`クラスとかと同じ感じで，`private`な内部クラスを抽象型として返します．

```cs
//元コード
[Enumeration]
partial class ClassSample
{
    public static partial ClassSample Key0(int num);
    public static partial ClassSample Key1(string str);
}

//生成コード
abstract partial class ClassSample
{
    public enum Case 
    {
        Key0, Key1
    }
    class __Key0 : ClassSample
    {
        public _Key0(int num) : base(Case.Key0)
        {
            this.num = num;
        }
        public readonly int num;
    }

    class __Key1 : ClassSample
    {
        public _Key1(string str) : base(Case.Key1)
        {
            this.str = str;
        }
        public readonly string str;
    }

    ClassSample(Case type)
    {
        this.Type = type;
    }

    public static partial ClassSample Key0(int num) => new __Key0(num);
    public static partial ClassSample Key1(string str) => new __Key1(str);

    public static bool Key0(ClassSample __self, out int num)
    {
        if(__self.Type == Case.Key0)
        {
            var instance = (__Key0)__self;
            num = instance.num;
            return true;
        }
        num = default;
        return false;
    }

    public static bool Key1(ClassSample __self, out string str)
    {
        if(__self.Type == Case.Key1)
        {
            var instance = (__Key1)__self;
            str = instance.str;
            return true;
        }
        str = default;
        return false;
    }

    public Case Type { get; }
}

static class ClassSample_Extension
{
    public static bool Key0(this ClassSample __self, out int num) => ClassSample.Key0(__self, out num);
    public static bool Key1(this ClassSample __self, out string str) => ClassSample.Key1(__self, out str);
}
```
超簡単ですね．それぞれのKeyに対応した内部クラスを生成して返しているだけです．

`Case`は生成しなくても型で分岐可能ですがswitchするのに内部型を外に漏らす必要があり，型switchはifの列になってしまってパフォーマンス的にも不利になるので`Case`を生成します．

クラスのフィールドをすべて読み取り専用にしましたが，別に外部に漏れないので必須ではありません．
ヒープアロケーションを減らしたいのなら，書き換え可能にしてインスタンスを内部でプールするとかしてもいいかもしれません.

### 構造体ベース実装
構造体において値を重ねる場合，C#に元々ある共用体を用います．ですが，[共用体で重ねられる型]({{site.baseurl}}{% link _posts/2022-03-18-uniontype.md %})には制限があります．

#### 注意点 &nbsp; 1 : &nbsp; 参照型の個数

なのでうまく重ねてやる必要があって，
- 型引数でないunmanaged型 / 参照型
    - 共用体にまとめる．
- 値型 / 型引数
    - 重ねずに並べる

共用体を構成するにあたっては，参照型と値が重なってしまわないようにいくつか注意が必要です．以下のような構造を考えると，
```cs
[StructLayout(LayoutKind.Explicit)]
struct A
{
    struct B
    {
        object obj;
        int num;// C.obj と重なってしまう
    }

    struct C
    {
        string str;
        object obj;// B.num と重なってしまう．
    }
    
    [FieldOffset(0)]
    B b;
    [FieldOffset(0)]
    C c;
}
```
`B.num`と`C.obj`が重なってしまいます．なのでダミーを用意して参照型の個数を合わせる必要があります．

```cs
[StructLayout(LayoutKind.Explicit)]
struct A
{
    struct B
    {
        object obj;
        object __dummy_0;// ここが C.obj と重なる

        int num;// C.obj と重ならない
    }

    struct C
    {
        string str;
        object obj;// ここは B.num と重ならない
    }
    
    [FieldOffset(0)]
    B b;
    [FieldOffset(0)]
    C c;
}
```
#### 注意点 &nbsp; 2 : &nbsp; フィールドの順序

参照はすべて同じ大きさですが，値型はそうではないので参照型と値型の順序にも気をつける必要があります．

```cs
[StructLayout(LayoutKind.Explicit)]
struct A
{
    struct B
    {
        int num32;
        object obj;// C.num64 と重なってしまう
    }

    struct C
    {
        long num64;// B.obj と重なってしまう
        object obj;
    }

    [FieldOffset(0)]
    B b;
    [FieldOffset(0)]
    C c;
}
```
`B.obj`と`C.num64`が重なってしまいます(~~Packによっては重ならないけど~~)．なので，先に参照型を並べ，その後ろに値型をならべる必要があります．

#### 注意点 &nbsp; 3 : &nbsp; ジェネリック型

ジェネリック型に`StructLayout(LauoutKind.Explicit)`を指定することはできません．
型引数をフィールドに含まなくても内部クラスもジェネリック型扱いされるので，
```cs
struct Container<T>
{
    [StructLayout(LayoutKind.Explicit)]
    struct Union
    {
        [FieldOffset(0)]
        object obj;
        [FieldOffset(0)]
        string str;
    }
    Union union;
    T generic;
}
```
`Container<T>.Union`は`T`を含まないですがそれでも`TypeLoadException`が投げられます．

#### サンプルと生成結果
これらの注意点を踏まえて型の生成結果を考えます．

```cs
//元コード
[Enumeration]
partial struct StructSample<T, TRef> where TRef: class 
{
    public static partial StructSample<T, TRef> Key0(long num, object obj);
    public static partial StructSample<T, TRef> Key1(int num, T value, TRef reference);
    public static partial StructSample<T, TRef> Key2(T generic0, T generic1);
}

//生成コード

//元コードの名前空間に内部型を含みたくないので専用の名前空間を用意する．
namespace Enumeration.Internals
{
    //ジェネリック型の内部に共用体は定義できないので外で定義する．
    [StructLayout(LayoutKind.Explicit)]
    struct StructSample_T_TRef_Union 
    {
        public struct __Key0
        {
            //参照型が先
            public object obj;
            object __dummy_0;//参照型の数合わせ用ダミー

            //値型は後
            public long num;
        }   

        public struct __Key1
        {
            public string str;
            public object reference;

            public int num;
        }

        public struct __Key2 
        {
            object __dummy_0;
            object __dummy_1;
        }

        [FieldOffset(0)]
        public __Key0 Key0;
        [FieldOffset(0)]
        public __Key1 Key1;
        [FieldOffset(0)]
        public __Key2 Key2;
    }
}


readonly partial struct StructSample<T, TRef> where TRef: class 
{
    public enum Case
    {
        Key0, Key1, Key2
    }

    struct __Implement
    {
        //重ねられない型は素直に並べる．
        public struct Serial
        {
            public T T_0;
            public T T_1;
        }

        public Enumeration.Internals.StructSample_T_TRef_Union union;
        public Serial serial;
        public Case type;

        //コンストラクタを呼ばずに変換する．ちょっとだけ早い．
        public StructSample<T, TRef> As() => Unsafe.As<__Implement, StructSample<T, TRef>>(ref this);
    }

    public static partial StructSample<T, TRef> Key0(long num, object obj)
    {
        var impl = default(__Implement);
        impl.type = Case.Key0;
        
        impl.union.Key0.num = num;
        impl.union.Key0.obj = obj;

        return impl.As();
    }
    public static partial StructSample<T, TRef> Key1(int num, T value, string str, TRef reference)
    {
        var impl = default(__Implement);
        impl.type = Case.Key1;

        impl.union.Key1.num = num;
        impl.serial.T_0 = value;
        impl.union.Key1.str = str;
        impl.union.Key1.reference = reference;

        return impl.As();
    }

    public static partial StructSample<T, TRef> Key2(T generic0, T generic1)
    {
        var impl = default(__Implement);
        impl.type = Case.Key2;

        impl.serial.T_0 = generic0;
        impl.serial.T_1 = generic1;

        return impl.As();
    }

    public static bool Key0(StructSample<T, TRef> __self, out long num, out object obj)
    {
        if(__self.Type == Case.Key0)
        {
            num = __self.__implement.union.Key0.num;
            obj = __self.__implement.union.Key0.obj;
            return true;
        }
        num = default;
        obj = default;
        return false;
    }

    public static bool Key1(StructSample<T, TRef> __self, out int num, out T value, out string str, out TRef reference)
    {
        if(__self.Type == Case.Key1)
        {
            num = __self.__implement.union.Key1.num;
            value = __self.__implement.serial.T_0;
            str = __self.__implement.union.Key1.str;
            reference = (TRef)__self.union.Key1.reference;
            return true;
        }
        num = default;
        value = default;
        str = default;
        reference = default;
        return false;
    }

    public static partial bool Key2(StructSample<T, TRef> __self, out T generic0, out T generic1)
    {
        if(__self.Type == Case.Key2)
        {
            generic0 = __self.__implement.serial.T_0;
            generic1 = __self.__implement.serial.T_1;
            return true;
        }
        generic0 = default;
        generic1 = default;
        return false;
    }

    readonly __Implement __implement;
    public Case Type => __implement.type;
}

static class StructSample_Extension
{
    public static bool Key0(this StructSample<T, TRef> __self, out long num, out object obj) 
    {
        return StructSample<T, TRef>.Key0(__self, out num, out obj);
    }
    public static bool Key1(this StructSample<T, TRef> __self, out int num, out T value, out string str, out TRef reference)
    {
        return StructSample<T, TRef>.Key1(__self, out num, out value, out str, out reference);
    }
    public static bool Key2(this StructSample<T, TRef> __self, out T generic0, out T generic1)
    {
        return StructSample<T, TRef>.Key2(__self, out num, out generic0, out generic1);
    }
}

```

かなり複雑になりましたね．参照型と比べるとかなりの差です．

更にこれから`IEquatable`だとか演算子だとかを加えて，シリアライズに対応して， nullableアノテーションつけて……

とまあ，ちゃんと作ろうとするとまだまだ追加する必要があるんですけども．

実装をすべて`__Implement`に投げているのは初期化を楽にするためです．ここまで複雑になると流石にコンストラクタだけで初期化は厳しいので可変がいいんですが，外から見て読み取り専用にしたいので，一枚はさんでいます．

複雑さを増しているのは型引数の扱い周りですが，結局ジェネリクスって型を生成するだけなのでもうちょっとカスタマイズさせてくれないかなとか思ったり．すでに非同期メソッドに関しては`AsyncMethodBuilder`属性でカスタム出来るし．
SourceGeneratorがあるんだからジェネリック型生成用のSourceGeneratorを指定したり出来たら，型引数の扱いが簡単になるんですけどね．

さて，ここからが本題で次はこれを生成するSourceGeneratorを作っていきます．


# ジェネレータの実装

結構複雑なことをしているので，純粋にC#だけでは厳しいです．大変なだけで書くことは当然できますが．なのでT4を使います．

詳しい説明は[こちら](https://neue.cc/2019/12/06_585.html)にお譲りします．SourceGeneratorの作りに関してもこの方のUnitGeneratorとかがとても参考になります．

今回は無印のSourceGeneratorではなくパフォーマンスが向上したIncrementalSourceGeneratorを作ります．
IDEによっては対応していないので無印SourceGeneratorも必要になるかもしれませんが，生成する対象を拾ってくる部分に違いが出るだけで他は同じです．

IncrementalSourceGeneratorに関しては[こちら](https://zenn.dev/pcysl5edgo/articles/6d9be0dd99c008)を参考にさせて頂きました．

## 前準備
T4はテンプレートファイルの中でC#のコードを好きに書くことができますが，`<# #>`が結構ノイズになるし，IntelliSenseとか無いので色々と結構つらいです．

生成されるテンプレートは部分型になっているので，複雑な処理はテンプレートファイルの外で書いていきます．生成に必要な情報もひとまとめにしてコンストラクタから渡してやると簡単です．

```cs
readonly struct EnumerationOptions
{
    static SymbolDisplayFormat NameOnly { get; } = new SymbolDisplayFormat(genericsOptions: SymbolDisplayGenericsOptions.IncludeTypeParameters | SymbolDisplayGenericsOptions.IncludeVariance);

    public EnumerationOptions(INamedTypeSymbol symbol)
    {
        var namespaceSymbol = symbol.ContainingNamespace;
        this.Namespace = namespaceSymbol.IsGlobalNamespace ? null : namespaceSymbol.ToDisplayString();
        this.Identifier = Helper.FullNameOf(symbol);
        this.Symbol = symbol;
        this.Name = symbol.ToDisplayString(NameOnly);
        var methods = symbol.GetMembers().OfType<IMethodSymbol>()
                            .Where(member => member.DeclaredAccessibility == Accessibility.Public && member.IsStatic && member.IsPartialDefinition && !member.IsGenericMethod)
                            .ToImmutableArray();
        this.Methods = methods;
        var syntax = (symbol.DeclaringSyntaxReferences.FirstOrDefault()?.GetSyntax() as TypeDeclarationSyntax) ?? throw new NotSupportedException();
        this.Syntax = syntax;
        this.TypeParameterConstraints = syntax.ConstraintClauses;

        var referenceTypeCount = 0;
        var types = new Dictionary<ITypeSymbol, (int Count, int Temp)>(SymbolEqualityComparer.Default);
        foreach (var method in methods)
        {
            var refCount = 0;
            foreach (var param in method.Parameters)
            {
                var type = param.Type;
                if (type.IsReferenceType)
                {
                    refCount++;
                    continue;
                }
                if (type is not ITypeParameterSymbol && type.IsUnmanagedType) continue;
                var contains = types.TryGetValue(type, out var pair);
                pair.Temp++;
                if (contains) types[type] = pair;
                else types.Add(type, pair);
            }
            if (referenceTypeCount < refCount) referenceTypeCount = refCount;

            var keys = types.Keys;
            for (var i = 0; i < keys.Count; ++i)
            {
                var key = keys.ElementAt(i);
                var (count, temp) = types[key];
                types[key] = (Math.Max(count, temp), 0);
            }
        }

        this.SerialTypes = types.Select(pair => (pair.Key, pair.Value.Count)).ToImmutableArray();
        this.ReferenceTypeCount = referenceTypeCount;
    }

    public ImmutableArray<(ITypeSymbol Type, int Count)> SerialTypes { get; }
    public int ReferenceTypeCount { get; }
    public string? Namespace { get; }
    public string Name { get; }
    public string Identifier { get; }
    public INamedTypeSymbol Symbol { get; }
    public ImmutableArray<IMethodSymbol> Methods { get; }
    public string AccessibilityString => SyntaxFacts.GetText(this.Symbol.DeclaredAccessibility);
    public bool IsNamespaceSpecified => !string.IsNullOrEmpty(this.Namespace);
    public TypeDeclarationSyntax Syntax { get; }
    public SyntaxList<TypeParameterConstraintClauseSyntax> TypeParameterConstraints { get; }


    public string DeconstructMethodSignatureOf(IMethodSymbol method)
    {
        var builder = new StringBuilder();
        builder.Append(method.Name);
        builder.Append('(');
        builder.Append(this.Identifier);
        builder.Append(" __self");
        var parameters = method.Parameters;
        foreach (var param in parameters)
        {
            builder.Append(", ");
            builder.Append(Helper.FullNameOf(param.Type));
            builder.Append(' ');
            builder.Append(param.Name);
        }
        builder.Append(')');
        return builder.ToString();
    }
}


static class Helper
{
    public static bool SymbolEquals(ISymbol? left, ISymbol? right)
    {
        return SymbolEqualityComparer.Default.Equals(left, right);
    }

    public static string ParamsOf(IMethodSymbol method) => string.Join(", ", method.Parameters.Select(p => $"{FullNameOf(p.Type)} {p.Name}"));
    public static string OutParamsOf(IMethodSymbol method) => string.Join(", ", method.Parameters.Select(p => $"out {FullNameOf(p.Type)} {p.Name}"));

    public static string FullNameOf(ISymbol symbol) => symbol.ToDisplayString(SymbolDisplayFormat.FullyQualifiedFormat);
    public static string EscapedFullNameOf(ISymbol symbol) => FullNameOf(symbol).Replace("global::", "").Replace(".", "_").Replace("<", "_").Replace(",", "_").Replace(" ", "").Replace(">", "");

    public static string IdentifierOf(IMethodSymbol method)
    {
        if (method.ContainingSymbol is not INamedTypeSymbol classSymbol) return method.Name;
        if (classSymbol.TypeParameters.IsEmpty) return method.Name;
        return $"{method.Name}<{string.Join(", ", classSymbol.TypeParameters)}>";
    }

    public static bool IsSerialType(ITypeSymbol type)
    {
        if (type.IsReferenceType) return false;
        if (type is not ITypeParameterSymbol && type.IsUnmanagedType) return false;
        return true;
    }
}
```

テンプレートは
- EnumerationAttributeTemplate.tt
- EnumerationStructTemplate.tt
- EnumerationClassTemplate.tt
- EnumerationExtensionTemplate.tt

の4個を作ります．
属性の内容は固定なのでT4を使わなくても良いですが，T4のほうがそのまま書けるのでちょっと楽です．
拡張メソッドは構造体，クラスで同じ内容なので分離して使いまわします．