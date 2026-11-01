<?xml version="1.0" encoding="UTF-8"?>
<sch:schema xmlns:sch="http://purl.oclc.org/dsdl/schematron" queryBinding="xslt2">
  <sch:ns prefix="f" uri="http://hl7.org/fhir"/>
  <sch:ns prefix="h" uri="http://www.w3.org/1999/xhtml"/>
  <!-- 
    This file contains just the constraints for the profile C4BBExplanationOfBenefit
    It includes the base constraints for the resource as well.
    Because of the way that schematrons and containment work, 
    you may need to use this schematron fragment to build a, 
    single schematron that validates contained resources (if you have any) 
  -->
  <sch:pattern>
    <sch:title>f:ExplanationOfBenefit</sch:title>
    <sch:rule context="f:ExplanationOfBenefit">
      <sch:assert test="count(f:diagnosis) &gt;= 1">diagnosis: minimum cardinality of 'diagnosis' is 1</sch:assert>
    </sch:rule>
  </sch:pattern>
  <sch:pattern>
    <sch:title>f:ExplanationOfBenefit/f:identifier</sch:title>
    <sch:rule context="f:ExplanationOfBenefit/f:identifier">
      <sch:assert test="count(f:type) &gt;= 1">type: minimum cardinality of 'type' is 1</sch:assert>
      <sch:assert test="count(f:value) &gt;= 1">value: minimum cardinality of 'value' is 1</sch:assert>
    </sch:rule>
  </sch:pattern>
  <sch:pattern>
    <sch:title>f:ExplanationOfBenefit/f:careTeam</sch:title>
    <sch:rule context="f:ExplanationOfBenefit/f:careTeam">
      <sch:assert test="count(f:role) &gt;= 1">role: minimum cardinality of 'role' is 1</sch:assert>
    </sch:rule>
  </sch:pattern>
  <sch:pattern>
    <sch:title>f:ExplanationOfBenefit/f:supportingInfo</sch:title>
    <sch:rule context="f:ExplanationOfBenefit/f:supportingInfo">
      <sch:assert test="count(f:timing[x]) &gt;= 1">timing[x]: minimum cardinality of 'timing[x]' is 1</sch:assert>
      <sch:assert test="count(f:code) &lt;= 0">code: maximum cardinality of 'code' is 0</sch:assert>
      <sch:assert test="count(f:timing[x]) &lt;= 0">timing[x]: maximum cardinality of 'timing[x]' is 0</sch:assert>
      <sch:assert test="count(f:value[x]) &gt;= 1">value[x]: minimum cardinality of 'value[x]' is 1</sch:assert>
      <sch:assert test="count(f:code) &lt;= 0">code: maximum cardinality of 'code' is 0</sch:assert>
      <sch:assert test="count(f:timing[x]) &lt;= 0">timing[x]: maximum cardinality of 'timing[x]' is 0</sch:assert>
      <sch:assert test="count(f:reason) &gt;= 1">reason: minimum cardinality of 'reason' is 1</sch:assert>
      <sch:assert test="count(f:code) &lt;= 0">code: maximum cardinality of 'code' is 0</sch:assert>
      <sch:assert test="count(f:timing[x]) &lt;= 0">timing[x]: maximum cardinality of 'timing[x]' is 0</sch:assert>
      <sch:assert test="count(f:value[x]) &gt;= 1">value[x]: minimum cardinality of 'value[x]' is 1</sch:assert>
      <sch:assert test="count(f:code) &lt;= 0">code: maximum cardinality of 'code' is 0</sch:assert>
      <sch:assert test="count(f:timing[x]) &lt;= 0">timing[x]: maximum cardinality of 'timing[x]' is 0</sch:assert>
      <sch:assert test="count(f:value[x]) &gt;= 1">value[x]: minimum cardinality of 'value[x]' is 1</sch:assert>
      <sch:assert test="count(f:code) &lt;= 0">code: maximum cardinality of 'code' is 0</sch:assert>
      <sch:assert test="count(f:timing[x]) &lt;= 0">timing[x]: maximum cardinality of 'timing[x]' is 0</sch:assert>
      <sch:assert test="count(f:value[x]) &gt;= 1">value[x]: minimum cardinality of 'value[x]' is 1</sch:assert>
      <sch:assert test="count(f:code) &lt;= 0">code: maximum cardinality of 'code' is 0</sch:assert>
      <sch:assert test="count(f:timing[x]) &lt;= 0">timing[x]: maximum cardinality of 'timing[x]' is 0</sch:assert>
      <sch:assert test="count(f:value[x]) &gt;= 1">value[x]: minimum cardinality of 'value[x]' is 1</sch:assert>
      <sch:assert test="count(f:code) &lt;= 0">code: maximum cardinality of 'code' is 0</sch:assert>
      <sch:assert test="count(f:timing[x]) &lt;= 0">timing[x]: maximum cardinality of 'timing[x]' is 0</sch:assert>
      <sch:assert test="count(f:value[x]) &gt;= 1">value[x]: minimum cardinality of 'value[x]' is 1</sch:assert>
    </sch:rule>
  </sch:pattern>
  <sch:pattern>
    <sch:title>f:ExplanationOfBenefit/f:supportingInfo/f:value[x] 1</sch:title>
    <sch:rule context="f:ExplanationOfBenefit/f:supportingInfo/f:value[x]">
      <sch:assert test="count(f:value) &gt;= 1">value: minimum cardinality of 'value' is 1</sch:assert>
      <sch:assert test="count(f:value) &gt;= 1">value: minimum cardinality of 'value' is 1</sch:assert>
    </sch:rule>
  </sch:pattern>
  <sch:pattern>
    <sch:title>f:ExplanationOfBenefit/f:diagnosis</sch:title>
    <sch:rule context="f:ExplanationOfBenefit/f:diagnosis">
      <sch:assert test="count(f:type) &gt;= 1">type: minimum cardinality of 'type' is 1</sch:assert>
      <sch:assert test="count(f:type) &lt;= 1">type: maximum cardinality of 'type' is 1</sch:assert>
    </sch:rule>
  </sch:pattern>
  <sch:pattern>
    <sch:title>f:ExplanationOfBenefit/f:item</sch:title>
    <sch:rule context="f:ExplanationOfBenefit/f:item">
      <sch:assert test="count(f:serviced[x]) &gt;= 1">serviced[x]: minimum cardinality of 'serviced[x]' is 1</sch:assert>
      <sch:assert test="count(f:location[x]) &gt;= 1">location[x]: minimum cardinality of 'location[x]' is 1</sch:assert>
    </sch:rule>
  </sch:pattern>
  <sch:pattern>
    <sch:title>f:ExplanationOfBenefit/f:item/f:adjudication</sch:title>
    <sch:rule context="f:ExplanationOfBenefit/f:item/f:adjudication">
      <sch:assert test="count(f:reason) &gt;= 1">reason: minimum cardinality of 'reason' is 1</sch:assert>
      <sch:assert test="count(f:reason) &gt;= 1">reason: minimum cardinality of 'reason' is 1</sch:assert>
    </sch:rule>
  </sch:pattern>
  <sch:pattern>
    <sch:title>f:ExplanationOfBenefit/f:adjudication</sch:title>
    <sch:rule context="f:ExplanationOfBenefit/f:adjudication">
      <sch:assert test="count(f:id) &lt;= 1">id: maximum cardinality of 'id' is 1</sch:assert>
      <sch:assert test="count(f:category) &gt;= 1">category: minimum cardinality of 'category' is 1</sch:assert>
      <sch:assert test="count(f:category) &lt;= 1">category: maximum cardinality of 'category' is 1</sch:assert>
      <sch:assert test="count(f:reason) &lt;= 1">reason: maximum cardinality of 'reason' is 1</sch:assert>
      <sch:assert test="count(f:amount) &lt;= 1">amount: maximum cardinality of 'amount' is 1</sch:assert>
      <sch:assert test="count(f:value) &lt;= 1">value: maximum cardinality of 'value' is 1</sch:assert>
      <sch:assert test="count(f:id) &lt;= 1">id: maximum cardinality of 'id' is 1</sch:assert>
      <sch:assert test="count(f:category) &gt;= 1">category: minimum cardinality of 'category' is 1</sch:assert>
      <sch:assert test="count(f:category) &lt;= 1">category: maximum cardinality of 'category' is 1</sch:assert>
      <sch:assert test="count(f:reason) &gt;= 1">reason: minimum cardinality of 'reason' is 1</sch:assert>
      <sch:assert test="count(f:reason) &lt;= 1">reason: maximum cardinality of 'reason' is 1</sch:assert>
      <sch:assert test="count(f:amount) &lt;= 1">amount: maximum cardinality of 'amount' is 1</sch:assert>
      <sch:assert test="count(f:value) &lt;= 1">value: maximum cardinality of 'value' is 1</sch:assert>
      <sch:assert test="count(f:id) &lt;= 1">id: maximum cardinality of 'id' is 1</sch:assert>
      <sch:assert test="count(f:category) &gt;= 1">category: minimum cardinality of 'category' is 1</sch:assert>
      <sch:assert test="count(f:category) &lt;= 1">category: maximum cardinality of 'category' is 1</sch:assert>
      <sch:assert test="count(f:reason) &gt;= 1">reason: minimum cardinality of 'reason' is 1</sch:assert>
      <sch:assert test="count(f:reason) &lt;= 1">reason: maximum cardinality of 'reason' is 1</sch:assert>
      <sch:assert test="count(f:amount) &lt;= 1">amount: maximum cardinality of 'amount' is 1</sch:assert>
      <sch:assert test="count(f:value) &lt;= 1">value: maximum cardinality of 'value' is 1</sch:assert>
    </sch:rule>
  </sch:pattern>
</sch:schema>
