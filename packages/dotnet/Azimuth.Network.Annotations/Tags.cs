using System;
namespace Azimuth.Network.Annotations
{
    [AttributeUsage(AttributeTargets.Class | AttributeTargets.Method, AllowMultiple = false)]
    public sealed class ImplementsProbeAttribute : Attribute
    {
        public ImplementsProbeAttribute(string probe) { Probe = probe; }
        public string Probe { get; }
    }
}
