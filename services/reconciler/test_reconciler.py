import copy
import unittest

from services.reconciler.reconciler import declared_template


class DeclaredTemplateTests(unittest.TestCase):
    def setUp(self):
        self.declared = {
            "metadata": {"labels": {"app": "example"}},
            "spec": {"containers": [{
                "name": "example",
                "image": "example@sha256:" + "a" * 64,
                "env": [{"name": "POD_NAME", "valueFrom": {
                    "fieldRef": {"fieldPath": "metadata.name"}
                }}],
            }]},
        }

    def test_admitted_field_reference_matches_declared_template(self):
        actual = copy.deepcopy(self.declared)
        actual["spec"]["containers"][0]["env"][0]["valueFrom"]["fieldRef"]["apiVersion"] = "v1"
        self.assertEqual(declared_template(actual, self.declared), self.declared)
        self.assertEqual(actual["spec"]["containers"][0]["env"][0]["valueFrom"]["fieldRef"]["apiVersion"], "v1")

    def test_changed_field_reference_remains_drift(self):
        actual = copy.deepcopy(self.declared)
        actual["spec"]["containers"][0]["env"][0]["valueFrom"]["fieldRef"] = {
            "apiVersion": "v1", "fieldPath": "metadata.namespace"
        }
        self.assertNotEqual(declared_template(actual, self.declared), self.declared)

    def test_explicit_field_version_mismatch_remains_drift(self):
        self.declared["spec"]["containers"][0]["env"][0]["valueFrom"]["fieldRef"]["apiVersion"] = "v1"
        actual = copy.deepcopy(self.declared)
        actual["spec"]["containers"][0]["env"][0]["valueFrom"]["fieldRef"]["apiVersion"] = "v2"
        self.assertNotEqual(declared_template(actual, self.declared), self.declared)


if __name__ == "__main__":
    unittest.main()
