import convert from "../src/convert";

describe('Convert JSON to ASLAN', () => {
  test('converts simple JSON to ASLAN', () => {
    const result = convert('{"name": "John", "age": 30}', 'aslan');
    expect(result).toEqual('[asland_name]John[asland_age]30');
  });

  test('converts nested JSON to ASLAN', () => {
    const result = convert('{"name": "John", "age": 30, "address": {"city": "New York", "state": "NY"}}', 'aslan');
    expect(result).toEqual('[asland_name]John[asland_age]30[asland_address][aslano][asland_city]New York[asland_state]NY');
  });

  test('converts array to ASLAN', () => {
    const result = convert('{"people": [{"name": "John", "age": 30}, {"name": "Jane", "age": 25}]}', 'aslan');
    expect(result).toEqual('[asland_people][aslana][asland][aslano][asland_name]John[asland_age]30[aslano][asland][aslano][asland_name]Jane[asland_age]25');
  });

  test('converts top level array to ASLAN', () => {
    const result = convert('[{"name": "John", "age": 30}, {"name": "Jane", "age": 25}]', 'aslan');
    expect(result).toEqual('[aslana][asland][aslano][asland_name]John[asland_age]30[aslano][asland][aslano][asland_name]Jane[asland_age]25');
  });
});
