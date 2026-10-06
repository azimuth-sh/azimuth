using System;
namespace Azimuth.Surface.Annotations
{
    [AttributeUsage(AttributeTargets.Class | AttributeTargets.Method, AllowMultiple = false)]
    public sealed class SurfaceEnumeratorAttribute : Attribute
    {
        public SurfaceEnumeratorAttribute(string surface) { Surface = surface; }
        public string Surface { get; }
    }
    [AttributeUsage(AttributeTargets.Class | AttributeTargets.Method, AllowMultiple = false)]
    public sealed class SurfaceExpectationsAttribute : Attribute
    {
        public SurfaceExpectationsAttribute(string expectations) { Expectations = expectations; }
        public string Expectations { get; }
        public string? ConfigurationFile { get; set; }
    }
}
