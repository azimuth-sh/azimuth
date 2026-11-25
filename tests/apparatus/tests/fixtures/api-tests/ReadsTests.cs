using System.Text.Json.Nodes;
using Xunit;

namespace Payouts.Api.Tests;

// Each test is one scenario for the reads the adapter collects. The tests of Azimuth ask for one
// Check id at a time, so each scenario runs alone in its launch.
public class ReadsTests
{
    [Fact]
    [Trait("Check", "host-gives-a-pair")]
    public void A_host_of_api_gives_a_built_and_a_disposing_read()
    {
        Reads.Report("api-built.json", "api", "built", Reads.Document("api", "built"));
        Reads.Report("api-disposing.json", "api", "disposing", Reads.Document("api", "disposing"));
    }

    [Fact]
    [Trait("Check", "host-gives-an-error")]
    public void A_host_of_api_gives_an_error_instead_of_a_report()
    {
        Reads.Report("api-built.json", "api", "built", Reads.Document("api", "built"));
        Reads.Error("api-disposing.json", "api", "disposing", "the host was disposed before it could be read");
    }

    [Fact]
    [Trait("Check", "hosts-give-several-pairs")]
    public void Two_hosts_of_api_and_a_separate_website_give_their_reads()
    {
        for (var host = 1; host <= 2; host++)
        {
            Reads.Report($"api-built-{host}.json", "api", "built", Reads.Document("api", $"built {host}"));
            Reads.Report($"api-disposing-{host}.json", "api", "disposing", Reads.Document("api", $"disposing {host}"));
        }
        Reads.Report("website-start.json", "website", "start", Reads.Document("website", "start"));
        Reads.Report("website-stop.json", "website", "stop", Reads.Document("website", "stop"));
    }

    [Fact]
    [Trait("Check", "host-of-an-unlisted-component")]
    public void A_host_of_a_component_the_request_does_not_list_gives_its_reads()
    {
        Reads.Report("billing-built.json", "billing", "built", Reads.Document("billing", "built"));
        Reads.Report("billing-disposing.json", "billing", "disposing", Reads.Document("billing", "disposing"));
    }

    [Fact]
    [Trait("Check", "website-is-a-separate-process")]
    public void A_website_started_by_the_tests_gives_a_start_and_a_stop_read()
    {
        var port = Environment.GetEnvironmentVariable("PAYOUTS_WEBSITE_PORT") ?? "none";
        Reads.Report("website-start.json", "website", "start", Reads.Document("website", $"start on port {port}"));
        Reads.Report("website-stop.json", "website", "stop", Reads.Document("website", $"stop on port {port}"));
    }

    // Writes into the report what this launch found in its folder before it wrote anything, so a
    // test of Azimuth can see the folder as the code of the tests saw it.
    [Fact]
    [Trait("Check", "tests-look-at-their-folder")]
    public void The_tests_record_what_they_find_in_the_folder()
    {
        var folder = Reads.Folder;
        var listing = new JsonArray();
        foreach (var file in Directory.EnumerateFileSystemEntries(folder).Select(Path.GetFileName).Order(StringComparer.Ordinal))
        {
            listing.Add(file);
        }
        var found = new JsonObject
        {
            ["folder"] = folder,
            ["exists"] = Directory.Exists(folder),
            ["listing"] = listing,
            ["components_json"] = File.Exists(Reads.ComponentsFile) ? File.ReadAllText(Reads.ComponentsFile) : null,
            ["names"] = new JsonArray(Reads.ComponentNames().Select(name => (JsonNode?)name).ToArray()),
        };
        Reads.Report("api-built.json", "api", "built", found);
        Reads.Report("api-disposing.json", "api", "disposing", Reads.Document("api", "disposing"));
    }

    [Fact]
    [Trait("Check", "tests-write-no-read")]
    public void The_tests_write_no_read()
    {
    }

    [Fact]
    [Trait("Check", "file-is-not-json")]
    public void A_file_that_is_not_JSON_stands_in_the_folder()
    {
        Reads.Report("api-built.json", "api", "built", Reads.Document("api", "built"));
        Reads.WriteText("notes.txt", "these are not the words of a read");
    }

    [Fact]
    [Trait("Check", "file-has-no-name")]
    public void A_file_with_no_name_stands_in_the_folder()
    {
        Reads.Write("nameless.json", new JsonObject { ["phase"] = "built", ["report"] = Reads.Document("api", "built") });
    }

    [Fact]
    [Trait("Check", "file-has-no-phase")]
    public void A_file_with_no_phase_stands_in_the_folder()
    {
        Reads.Write("phaseless.json", new JsonObject { ["name"] = "api", ["report"] = Reads.Document("api", "built") });
    }

    [Fact]
    [Trait("Check", "name-is-not-text")]
    public void A_file_whose_name_is_not_text_stands_in_the_folder()
    {
        Reads.Write("numbered.json", new JsonObject { ["name"] = 7, ["phase"] = "built", ["report"] = Reads.Document("api", "built") });
    }

    [Fact]
    [Trait("Check", "phase-is-not-text")]
    public void A_file_whose_phase_is_not_text_stands_in_the_folder()
    {
        Reads.Write("unphased.json", new JsonObject { ["name"] = "api", ["phase"] = true, ["report"] = Reads.Document("api", "built") });
    }
}
