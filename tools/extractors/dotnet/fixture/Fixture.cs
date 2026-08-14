using System;
using System.Threading.Tasks;
using Azimuth.Annotations;

namespace Azimuth.Fixture
{
    /// A type-level tag names its site by the type; a method-level tag by Type.Method.
    [Realizes("alpha", "type-level-thing")]
    public sealed class Production
    {
        [Realizes("alpha", "method-level-thing")]
        public void Method()
        {
        }

        /// A site may realize several claims — branches of one method.
        [Realizes("alpha", "first-branch")]
        [Realizes("alpha", "second-branch")]
        [ImplementsMechanism("alpha", "branch-selection")]
        public void Branching()
        {
        }

        public void Untagged()
        {
        }

        /// An async method's sequence points live on the state machine's MoveNext, not here.
        [Realizes("alpha", "async-thing")]
        public async Task AsyncWork()
        {
            await Task.Yield();
        }
    }

    /// A value type is a realization site too — and the type that *is* an enforcement mechanism is
    /// often a struct, so excluding them would have made the strongest rung untaggable.
    [Realizes("alpha", "struct-level-thing")]
    public readonly record struct Amount(long MinorUnits);

    /// A type with no ordinary methods still needs a navigable realization source. Validators and
    /// configuration types commonly put their entire predicate in the constructor.
    [Realizes("alpha", "constructor-only-thing")]
    public sealed class ConstructorOnly
    {
        public ConstructorOnly()
        {
            Value = 1;
        }

        public int Value { get; }
    }

    /// Stands in for a test framework attribute. Matched by simple name, so the emitter needs no
    /// reference to any test framework.
    [AttributeUsage(AttributeTargets.Method)]
    public sealed class FactAttribute : Attribute
    {
    }
}

namespace Azimuth.Fixture.Traced
{
    using Azimuth.Fixture;

    public sealed class Tests
    {
        [Fact]
        [Covers("alpha", "method-level-thing", Scope.Component, Quantification.Universal)]
        public void Covered()
        {
        }

        [Fact]
        [Covers("alpha", "relational-thing", Scope.Component, Quantification.Universal,
            Oracle.Relational)]
        public void CoveredRelationally()
        {
        }

        [Fact]
        [Covers("alpha", "first-branch", Scope.E2e, Quantification.Example, Oracle.ModelBased)]
        [CoversMechanism(
            "alpha",
            "branch-selection",
            Scope.Component,
            Quantification.Universal,
            Oracle.ModelBased)]
        public void CoveredWithOracle()
        {
        }

        [Fact]
        public void SmokeCheck()
        {
        }

        [Fact]
        public void Bare()
        {
        }
    }
}

namespace Azimuth.Fixture.Ordinary
{
    using Azimuth.Fixture;

    public sealed class Tests
    {
        [Fact]
        public void BareButOutsideAnyTracedRoot()
        {
        }
    }
}
